<script setup lang="ts">
  import { userActionsKey } from '../data/keys'
  import type { User } from '../data/schema'
  import type { DataTableRow } from '@/components/data-table'
  import { Button } from '@/components/ui/button'
  import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuItem,
    DropdownMenuSeparator,
    DropdownMenuShortcut,
    DropdownMenuTrigger,
  } from '@/components/ui/dropdown-menu'
  import { EllipsisIcon, Trash2, UserPen } from '@lucide/vue'
  import { inject } from 'vue'

  interface DataTableRowActionsProps {
    row: DataTableRow<User>
  }
  const props = defineProps<DataTableRowActionsProps>()

  const actions = inject(userActionsKey)
  const { openUserEditDialog } = actions!
</script>

<template>
  <div>
    <DropdownMenu>
      <DropdownMenuTrigger as-child>
        <Button
          variant="ghost"
          class="flex h-8 w-8 p-0 data-[state=open]:bg-muted"
        >
          <EllipsisIcon class="size-4" />
          <span class="sr-only">Open menu</span>
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent
        align="end"
        class="w-40"
      >
        <DropdownMenuItem @click="openUserEditDialog(props.row.original)">
          Edit
          <DropdownMenuShortcut>
            <UserPen :size="16" />
          </DropdownMenuShortcut>
        </DropdownMenuItem>
        <DropdownMenuSeparator />
        <DropdownMenuItem>
          {{ row.original.active ? 'Deactivate' : 'Activate' }}
          <DropdownMenuShortcut>
            <Trash2 :size="16" />
          </DropdownMenuShortcut>
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  </div>
</template>
