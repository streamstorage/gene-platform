<script setup lang="ts" generic="T extends RowData">
  import type { DataTableInstance } from '.'
  import { Button } from '@/components/ui/button'
  import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
  } from '@/components/ui/select'
  import { PAGE_SIZES } from '@/constants/app'
  import { cn, getPageNumbers } from '@/lib/utils'
  import { ChevronLeft, ChevronRight, ChevronsLeft, ChevronsRight } from '@lucide/vue'
  import type { RowData } from '@tanstack/vue-table'
  import { type HTMLAttributes, computed } from 'vue'

  type DataTablePaginationProps<T extends RowData> = {
    table: DataTableInstance<T>
    class?: HTMLAttributes['class']
  }
  const props = defineProps<DataTablePaginationProps<T>>()

  const pagination = computed(() => props.table.atoms.pagination.get())
  const currentPage = computed(() => pagination.value.pageIndex + 1)
  const totalPages = computed(() => props.table.getPageCount())
  const pageNumbers = computed(() => getPageNumbers(currentPage.value, totalPages.value))

  function handlePageSizeChange(value: unknown) {
    if (typeof value !== 'string' && typeof value !== 'number') return

    const newPageSize = Number(value)
    if (Number.isFinite(newPageSize) && newPageSize > 0) props.table.setPageSize(newPageSize)
  }
</script>

<template>
  <div
    :class="
      cn(
        'flex items-center justify-between overflow-clip px-2',
        '@max-2xl/content:flex-col-reverse @max-2xl/content:gap-4',
        props.class
      )
    "
    style="overflow-clip-margin: 1"
  >
    <div class="flex w-full items-center justify-between">
      <div class="flex w-25 items-center justify-center text-sm font-medium @2xl/content:hidden">
        {{ `Page ${currentPage} of ${totalPages}` }}
      </div>
      <div class="flex items-center gap-2 @max-2xl/content:flex-row-reverse">
        <Select
          :mode-value="`${pagination.pageSize}`"
          @update:model-value="handlePageSizeChange"
        >
          <SelectTrigger class="h-8 w-18">
            <SelectValue :placeholder="`${pagination.pageSize}`" />
          </SelectTrigger>
          <SelectContent
            side="top"
            @close-auto-focus="(event) => event.preventDefault()"
          >
            <SelectItem
              v-for="pageSize in PAGE_SIZES"
              :key="pageSize"
              :value="`${pageSize}`"
            >
              {{ pageSize }}
            </SelectItem>
          </SelectContent>
        </Select>
        <p class="hidden text-sm font-medium sm:block">Rows per page</p>
      </div>
    </div>

    <div class="flex items-center sm:space-x-6 lg:space-x-8">
      <div
        class="flex w-25 items-center justify-center text-sm font-medium @max-3xl/content:hidden"
      >
        {{ `Page ${currentPage} of ${totalPages}` }}
      </div>
      <div class="flex items-center space-x-2">
        <Button
          variant="outline"
          class="size-8 p-0 @max-md/content:hidden"
          @click="props.table.firstPage()"
          :disabled="!props.table.getCanPreviousPage()"
        >
          <span class="sr-only">Go to first page</span>
          <ChevronsLeft class="h-4 w-4" />
        </Button>
        <Button
          variant="outline"
          class="size-8 p-0"
          @click="props.table.previousPage()"
          :disabled="!props.table.getCanPreviousPage()"
        >
          <span class="sr-only">Go to previous page</span>
          <ChevronLeft class="h-4 w-4" />
        </Button>

        <div
          v-for="(pageNumber, index) in pageNumbers"
          :key="`${pageNumber}-${index}`"
          class="flex items-center"
        >
          <span
            v-if="pageNumber === '...'"
            class="px-1 text-sm text-muted-foreground"
            >...</span
          >
          <template v-else>
            <Button
              :variant="currentPage === pageNumber ? 'default' : 'outline'"
              class="h-8 min-w-8 px-2"
              @click="props.table.setPageIndex((pageNumber as number) - 1)"
            >
              <span class="sr-only">{{ `Go to page ${pageNumber}` }}</span>
              {{ pageNumber }}
            </Button>
          </template>
        </div>

        <Button
          variant="outline"
          class="size-8 p-0"
          @click="props.table.nextPage()"
          :disabled="!props.table.getCanNextPage()"
        >
          <span class="sr-only">Go to next page</span>
          <ChevronRight class="h-4 w-4" />
        </Button>
        <Button
          variant="outline"
          class="size-8 p-0 @max-md/content:hidden"
          @click="props.table.lastPage()"
          :disabled="!props.table.getCanNextPage()"
        >
          <span class="sr-only">Go to last page</span>
          <ChevronsRight class="h-4 w-4" />
        </Button>
      </div>
    </div>
  </div>
</template>
