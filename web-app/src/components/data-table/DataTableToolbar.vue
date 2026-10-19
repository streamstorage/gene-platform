<script setup lang="ts" generic="T extends RowData">
  import type { DataTableInstance } from '.'
  import DataTableFacetedFilter from './DataTableFacetedFilter.vue'
  import DataTableViewOptions from './DataTableViewOptions.vue'
  import { Button } from '@/components/ui/button'
  import { Input } from '@/components/ui/input'
  import { XIcon } from '@lucide/vue'
  import type { RowData } from '@tanstack/vue-table'
  import { type Component, computed } from 'vue'

  type DataTableToolbarProps<T extends RowData> = {
    table: DataTableInstance<T>
    searchPlaceholder?: string
    searchKey?: string
    filters?: {
      columnId: string
      title: string
      options: {
        label: string
        value: unknown
        icon?: Component
      }[]
    }[]
  }

  const props = withDefaults(defineProps<DataTableToolbarProps<T>>(), {
    searchPlaceholder: 'Filter...',
    filters: () => [],
  })

  const columnFilters = computed(() => props.table.atoms.columnFilters.get())
  const globalFilter = computed(() => props.table.atoms.globalFilter.get())

  const isFiltered = computed(() => columnFilters.value.length > 0 || globalFilter.value)
</script>

<template>
  <div class="flex items-center justify-between">
    <div
      class="flex flex-1 flex-col-reverse items-start gap-y-2 sm:flex-row sm:items-center sm:space-x-2"
    >
      <Input
        v-if="props.searchKey"
        :placeholder="props.searchPlaceholder"
        :model-value="(props.table.getColumn(props.searchKey)?.getFilterValue() as string) ?? ''"
        @input="
          (event: Event) =>
            props.table
              .getColumn(props.searchKey!)
              ?.setFilterValue((event.target as HTMLInputElement).value)
        "
        class="h-8 w-37.5 lg:w-62.5"
      />
      <Input
        v-else
        :placeholder="props.searchPlaceholder"
        :model-value="globalFilter ?? ''"
        @input="
          (event: Event) => props.table.setGlobalFilter((event.target as HTMLInputElement).value)
        "
        class="h-8 w-37.5 lg:w-62.5"
      />
      <div class="flex gap-x-2">
        <template
          v-for="filter in props.filters"
          :key="filter.columnId"
        >
          <DataTableFacetedFilter
            v-if="props.table.getColumn(filter.columnId)"
            :column="props.table.getColumn(filter.columnId)"
            :title="filter.title"
            :options="filter.options"
          />
        </template>
      </div>
      <Button
        v-if="isFiltered"
        variant="ghost"
        @click="
          () => {
            props.table.resetColumnFilters()
            props.table.setGlobalFilter('')
          }
        "
        class="h-8 px-2 lg:px-3"
      >
        Reset
        <XIcon class="ms-2 h-4 w-4" />
      </Button>
    </div>
    <DataTableViewOptions :table="table" />
  </div>
</template>
