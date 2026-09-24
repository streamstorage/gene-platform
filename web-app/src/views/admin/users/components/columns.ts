import { callTypes, roles } from '../data/data'
import { type User } from '../data/schema'
import RowActions from './RowActions.vue'
import type { DataTableColumnDef } from '@/components/data-table'
import { DataTableColumnHeader } from '@/components/data-table'
import { Badge } from '@/components/ui/badge'
import { Checkbox } from '@/components/ui/checkbox'
import { cn } from '@/lib/utils'
import { h } from 'vue'

export const usersColumns: DataTableColumnDef<User>[] = [
  {
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
  },
  {
    id: 'rowNumber',
    header: ({ column }) => h(DataTableColumnHeader<User>, { column, title: '#' }),
    cell: ({ row }) => h('div', {}, () => row.getDisplayIndex() + 1),
    enableHiding: false,
    enableSorting: false,
  },
  {
    accessorKey: 'name',
    header: ({ column }) => h(DataTableColumnHeader<User>, { column, title: 'Name' }),
    cell: ({ row }) => h('div', {}, () => row.getValue('name')),
    enableHiding: false,
  },
  {
    accessorKey: 'email',
    header: ({ column }) => h(DataTableColumnHeader<User>, { column, title: 'Email' }),
    cell: ({ row }) =>
      h(
        'div',
        {
          class: 'w-fit ps-2 text-nowrap',
        },
        row.getValue('email')
      ),
    enableHiding: false,
  },
  {
    accessorKey: 'notes',
    header: ({ column }) => h(DataTableColumnHeader<User>, { column, title: 'Notes' }),
    cell: ({ row }) => h('div', {}, () => row.getValue('notes')),
    enableSorting: false,
  },
  {
    accessorKey: 'status',
    header: ({ column }) => h(DataTableColumnHeader<User>, { column, title: 'Status' }),
    cell: ({ row }) => {
      const { status } = row.original
      const badgeColor = callTypes.get(status)
      return h(
        'div',
        { class: 'flex space-x-2' },
        h(
          Badge,
          {
            variant: 'outline',
            class: cn('capitalize', badgeColor),
          },
          () => row.getValue('status')
        )
      )
    },
    // filterFn: (row, id, value) => {
    //   return value.includes(row.getValue(id))
    // },
    filterFn: 'arrHas',
    enableHiding: false,
    enableSorting: false,
  },
  {
    accessorKey: 'role',
    header: ({ column }) => h(DataTableColumnHeader<User>, { column, title: 'Role' }),
    cell: ({ row }) => {
      const { role } = row.original
      const userType = roles.find(({ value }) => value === role)

      if (!userType) {
        return null
      }

      return h('div', { class: 'flex items-center gap-x-2' }, [
        userType.icon && h(userType.icon, { class: 'text-muted-foreground', size: 16 }),
        h('span', { class: 'text-sm capitalize' }, () => row.getValue('role')),
      ])
    },
    // filterFn: (row, id, value) => {
    //   return value.includes(row.getValue(id))
    // },
    filterFn: 'arrHas',
    enableSorting: false,
    enableHiding: false,
  },
  {
    id: 'actions',
    cell: ({ row }) => h(RowActions, { row }),
  },
]
