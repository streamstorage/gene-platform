<script setup lang="ts" generic="T extends RowData">
  import { userActionsKey } from '../data/keys'
  import { type User } from '../data/schema'
  import { type DataTableInstance, DataTableBulkActions } from '@/components/data-table'
  import { Button } from '@/components/ui/button'
  import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip'
  import { UserX, UserCheck } from '@lucide/vue'
  import type { RowData } from '@tanstack/vue-table'
  import { computed, inject } from 'vue'

  interface BulkActionsProps<T extends RowData> {
    table: DataTableInstance<T>
  }
  const { table } = defineProps<BulkActionsProps<T>>()

  const selectedRows = computed(() => table.getFilteredSelectedRowModel().rows)

  const actions = inject(userActionsKey)
  const { openUserActivateDialog } = actions!

  const handleBulkStatusChange = (state: boolean) => {
    const selectedUsers = selectedRows.value.map((row) => row.original as User)
    openUserActivateDialog(state, selectedUsers)
    table.resetRowSelection()
  }
</script>

<template>
  <DataTableBulkActions
    :table="table"
    entity-name="user"
  >
    <Tooltip>
      <TooltipTrigger as-child>
        <Button
          variant="outline"
          size="icon"
          @click="handleBulkStatusChange(true)"
          class="size-8"
          aria-label="Activate selected users"
          title="Activate selected users"
        >
          <UserCheck />
          <span className="sr-only">Activate selected users</span>
        </Button>
      </TooltipTrigger>
      <TooltipContent>
        <p>Activate selected users</p>
      </TooltipContent>
    </Tooltip>

    <Tooltip>
      <TooltipTrigger asChild>
        <Button
          variant="outline"
          size="icon"
          @click="handleBulkStatusChange(false)"
          class="size-8"
          aria-label="Deactivate selected users"
          title="Deactivate selected users"
        >
          <UserX />
          <span class="sr-only">Deactivate selected users</span>
        </Button>
      </TooltipTrigger>
      <TooltipContent>
        <p>Deactivate selected users</p>
      </TooltipContent>
    </Tooltip>
  </DataTableBulkActions>
</template>
