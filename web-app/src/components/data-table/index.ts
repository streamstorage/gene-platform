import { Checkbox } from '@/components/ui/checkbox'
import { cn } from '@/lib/utils'
import type { Column, ColumnDef, Row, RowData, Table } from '@tanstack/vue-table'
import {
  createFacetedRowModel,
  createFacetedUniqueValues,
  columnFacetingFeature,
  columnFilteringFeature,
  columnVisibilityFeature,
  globalFilteringFeature,
  rowSelectionFeature,
  rowPaginationFeature,
  rowSortingFeature,
  filterFn_arrHas,
  filterFn_includesString,
  sortFn_alphanumeric,
  sortFn_datetime,
  sortFn_text,
  createFilteredRowModel,
  createPaginatedRowModel,
  createSortedRowModel,
  tableFeatures,
} from '@tanstack/vue-table'
import { h } from 'vue'

export type ColumnFilters = Array<
  | {
      columnId: string
      searchKey: string
      type?: 'string'
      // Optional transformers for custom types
      serialize?: (value: unknown) => unknown
      deserialize?: (value: unknown) => unknown
    }
  | {
      columnId: string
      searchKey: string
      type: 'array'
      serialize?: (value: unknown) => unknown
      deserialize?: (value: unknown) => unknown
    }
>

export const features = tableFeatures({
  columnFacetingFeature,
  facetedRowModel: createFacetedRowModel(),
  facetedUniqueValues: createFacetedUniqueValues(),

  columnFilteringFeature,
  globalFilteringFeature,
  filteredRowModel: createFilteredRowModel(),

  columnVisibilityFeature,

  rowPaginationFeature,
  paginatedRowModel: createPaginatedRowModel(),

  rowSelectionFeature,
  rowSortingFeature,
  sortedRowModel: createSortedRowModel(),

  filterFns: {
    arrHas: filterFn_arrHas,
    includesString: filterFn_includesString,
  },
  sortFns: {
    alphanumeric: sortFn_alphanumeric,
    datetime: sortFn_datetime,
    text: sortFn_text,
  },
})

export type DataTableColumn<T extends RowData, TValue = unknown> = Column<
  typeof features,
  T,
  TValue
>
export type DataTableColumnDef<T extends RowData, TValue = unknown> = ColumnDef<
  typeof features,
  T,
  TValue
>
export type DataTableInstance<T extends RowData> = Table<typeof features, T>
export type DataTableRow<T extends RowData> = Row<typeof features, T>

// To use global filter, unset 'searchKey' in DataTableToolbar
export interface DataTableProps<T extends RowData> {
  columns: DataTableColumnDef<T>[]
  data: readonly T[]
  loading?: boolean
  columnFilters?: ColumnFilters
  enableGlobalFilter?: boolean
}

export function createSelectColumn<T extends RowData>(): DataTableColumnDef<T> {
  return {
    id: 'select',
    header: ({ table }) =>
      h(Checkbox, {
        modelValue:
          table.getIsAllPageRowsSelected() ||
          (table.getIsSomePageRowsSelected() && 'indeterminate'),
        'onUpdate:modelValue': (value) => table.toggleAllPageRowsSelected(!!value),
        ariaLabel: 'Select all',
        class: 'translate-y-0.5',
      }),
    meta: {
      class: cn('inset-s-0 z-10 rounded-tl-[inherit] max-md:sticky'),
    },
    cell: ({ row }) =>
      h(Checkbox, {
        modelValue: row.getIsSelected(),
        'onUpdate:modelValue': (value) => row.toggleSelected(!!value),
        'aria-label': 'Select row',
        class: 'translate-y-0.5',
      }),
    enableSorting: false,
    enableHiding: false,
  }
}

export { default as DataTable } from './DataTable.vue'
export { default as DataTableBulkActions } from './DataTableBulkActions.vue'
export { default as DataTableColumnHeader } from './DataTableColumnHeader.vue'
export { default as DataTableToolbar } from './DataTableToolbar.vue'
