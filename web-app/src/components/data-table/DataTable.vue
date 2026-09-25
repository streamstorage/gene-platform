<script setup lang="ts" generic="T extends RowData">
  import { type DataTableProps, type DataTableInstance, features } from '.'
  import DataTablePagination from './DataTablePagination.vue'
  import {
    Table,
    TableBody,
    TableCell,
    TableHead,
    TableHeader,
    TableRow,
  } from '@/components/ui/table'
  import { DEFAULT_PAGE_SIZE } from '@/constants/app'
  import { cn } from '@/lib/utils'
  import type {
    ColumnFiltersState,
    OnChangeFn,
    RowData,
    PaginationState,
  } from '@tanstack/vue-table'
  import { FlexRender, useTable } from '@tanstack/vue-table'
  import { ref, toRef, watch } from 'vue'
  import { useRoute, useRouter } from 'vue-router'

  defineOptions({ inheritAttrs: false })

  defineSlots<{
    'bulk-actions': (props: { table: DataTableInstance<T> }) => unknown
    toolbar: (props: { table: DataTableInstance<T> }) => unknown
  }>()

  const props = defineProps<DataTableProps<T>>()

  const columnFiltersCfg = props.columnFilters ?? []

  const pageKey = 'page'
  const pageSizeKey = 'pageSize'
  const defaultPage = 1
  const defaultPageSize = DEFAULT_PAGE_SIZE
  const globalFilterKey = 'filter'
  const globalFilterEnabled = props.enableGlobalFilter
  const trimGlobal = true

  type SearchRecord = Record<string, unknown>
  type QueryRecord = Record<string, string>

  const router = useRouter()
  const route = useRoute()
  const search = route.query

  const query = ref({
    ...(search as SearchRecord),
  })

  // Build initial column filters from the current search params
  const collected: ColumnFiltersState = []
  for (const cfg of columnFiltersCfg) {
    const raw = (search as SearchRecord)[cfg.searchKey]
    const deserialize = cfg.deserialize ?? ((v: unknown) => v)
    if (cfg.type === 'string') {
      const value = (deserialize(raw) as string) ?? ''
      if (typeof value === 'string' && value.trim() !== '') {
        collected.push({ id: cfg.columnId, value })
      }
    } else {
      // default to array type
      const array = raw ? (Array.isArray(raw) ? raw : [raw]) : undefined
      const value = (deserialize(array) as unknown[]) ?? []
      if (Array.isArray(value) && value.length > 0) {
        collected.push({ id: cfg.columnId, value })
      }
    }
  }
  const columnFilters = ref<ColumnFiltersState>(collected)

  // Init pagination
  const rawPage = (search as QueryRecord)[pageKey]
  const rawPageSize = (search as QueryRecord)[pageSizeKey]
  const pageNum = Number(rawPage) || defaultPage
  const pageSizeNum = Number(rawPageSize) || defaultPageSize
  const pagination = ref<PaginationState>({
    pageIndex: Math.max(0, pageNum - 1),
    pageSize: pageSizeNum,
  })

  const onPaginationChange: OnChangeFn<PaginationState> = (updater) => {
    const next = typeof updater === 'function' ? updater(pagination.value) : updater
    const nextPage = next.pageIndex + 1
    const nextPageSize = next.pageSize
    pagination.value = next

    query.value = {
      ...query.value,
      [pageKey]: nextPage <= defaultPage ? undefined : nextPage,
      [pageSizeKey]: nextPageSize === defaultPageSize ? undefined : nextPageSize,
    }
    router.push({
      path: route.path,
      query: { ...(query.value as QueryRecord) },
    })
  }

  let gf: string | undefined = undefined
  if (globalFilterEnabled) {
    const raw = (search as SearchRecord)[globalFilterKey]
    gf = typeof raw === 'string' ? raw : ''
  }
  const globalFilter = ref<string | undefined>(gf)

  const onGlobalFilterChange: OnChangeFn<string> | undefined = globalFilterEnabled
    ? (updater) => {
        const next = typeof updater === 'function' ? updater(globalFilter.value ?? '') : updater
        const value = trimGlobal ? next.trim() : next
        globalFilter.value = value
        pagination.value.pageIndex = 0
        query.value = {
          ...query.value,
          [pageKey]: undefined,
          [globalFilterKey]: value ? value : undefined,
        }
        router.push({
          path: route.path,
          query: { ...(query.value as QueryRecord) },
        })
      }
    : undefined

  const onColumnFiltersChange: OnChangeFn<ColumnFiltersState> = (updater) => {
    const next = typeof updater === 'function' ? updater(columnFilters.value) : updater
    columnFilters.value = next

    const patch: Record<string, unknown> = {}

    for (const cfg of columnFiltersCfg) {
      const found = next.find((f) => f.id === cfg.columnId)
      const serialize = cfg.serialize ?? ((v: unknown) => v)
      if (cfg.type === 'string') {
        const value = typeof found?.value === 'string' ? (found.value as string) : ''
        patch[cfg.searchKey] = value.trim() !== '' ? serialize(value) : undefined
      } else {
        const value = Array.isArray(found?.value) ? (found!.value as unknown[]) : []
        patch[cfg.searchKey] = value.length > 0 ? serialize(value) : undefined
      }
    }
    pagination.value.pageIndex = 0
    query.value = {
      ...query.value,
      [pageKey]: undefined,
      ...(patch as QueryRecord),
    }
    router.push({
      path: route.path,
      query: { ...(query.value as QueryRecord) },
    })
  }

  const ensurePageInRange = (
    table: DataTableInstance<T>,
    opts: { resetTo?: 'first' | 'last' } = { resetTo: 'last' }
  ) => {
    const pageCount = table.getPageCount()
    const currentPage = (search as QueryRecord)[pageKey]
    const pageNum = Number(currentPage) || defaultPage
    if (pageCount > 0 && pageNum > pageCount) {
      if (opts.resetTo === 'last') {
        table.lastPage()
      } else {
        table.firstPage()
      }
      query.value = {
        ...query.value,
        [pageKey]: opts.resetTo === 'last' ? pageCount : undefined,
      }
      router.replace({
        path: route.path,
        query: { ...(query.value as QueryRecord) },
      })
    }
  }

  const table = useTable({
    data: toRef(props, 'data'),
    columns: props.columns,
    features,
    enableRowSelection: true,
    autoResetPageIndex: false,
    state: {
      get columnFilters() {
        return columnFilters.value
      },
      get globalFilter() {
        return globalFilter.value
      },
      get pagination() {
        return pagination.value
      },
    },
    onColumnFiltersChange,
    onGlobalFilterChange,
    onPaginationChange,
  })

  const loading = toRef(props, 'loading')

  watch(
    () => table.getRowCount(),
    (val) => {
      if (val) {
        ensurePageInRange(table)
      }
    }
  )
</script>

<template>
  <div
    :class="
      cn(
        `max-sm:has-[div[role='toolbar']]:mb-16`, // Add margin bottom to the table on mobile when the toolbar is visible
        'flex flex-1 flex-col gap-4'
      )
    "
  >
    <slot
      v-if="$slots.toolbar"
      name="toolbar"
      :table="table"
    />
    <div class="overflow-hidden rounded-md border">
      <Table
        v-bind="$attrs"
        :aria-busy="loading || undefined"
      >
        <TableHeader>
          <TableRow
            v-for="headerGroup in table.getHeaderGroups()"
            :key="headerGroup.id"
            class="group/row"
          >
            <TableHead
              v-for="header in headerGroup.headers"
              :key="header.id"
              :colspan="header.colSpan"
              :class="
                cn(
                  'bg-background group-hover/row:bg-muted group-data-[state=selected]/row:bg-muted',
                  header.column.columnDef.meta?.class,
                  header.column.columnDef.meta?.thClass
                )
              "
            >
              <FlexRender
                v-if="!header.isPlaceholder"
                :header="header"
              />
            </TableHead>
          </TableRow>
        </TableHeader>
        <TableBody v-if="!loading">
          <template v-if="table.getRowModel().rows?.length">
            <TableRow
              v-for="row in table.getRowModel().rows"
              :key="row.id"
              :data-state="row.getIsSelected() && 'selected'"
              class="group/row"
            >
              <TableCell
                v-for="cell in row.getVisibleCells()"
                :key="cell.id"
                :class="
                  cn(
                    'bg-background group-hover/row:bg-muted group-data-[state=selected]/row:bg-muted',
                    cell.column.columnDef.meta?.class,
                    cell.column.columnDef.meta?.tdClass
                  )
                "
              >
                <FlexRender :cell="cell" />
              </TableCell>
            </TableRow>
          </template>
          <TableRow v-else>
            <TableCell
              :colspan="table.getAllColumns().length"
              class="h-24 text-center"
            >
              No results.
            </TableCell>
          </TableRow>
        </TableBody>
        <TableBody v-else>
          <TableRow>
            <TableCell
              :colspan="table.getAllColumns().length"
              class="h-24 text-center shimmer"
            >
              Loading…
            </TableCell>
          </TableRow>
        </TableBody>
      </Table>
    </div>
    <DataTablePagination
      v-if="!loading && table.getRowCount()"
      :table="table"
      class="mt-auto"
    />
    <slot
      v-if="$slots['bulk-actions']"
      name="bulk-actions"
      :table="table"
    />
  </div>
</template>
