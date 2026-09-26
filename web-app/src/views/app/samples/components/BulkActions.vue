<script setup lang="ts" generic="T extends RowData">
  import UserMultiDeleteDialog from './UserMultiDeleteDialog.vue'
  import { type User } from '../data/schema'
  import { type DataTableInstance, DataTableBulkActions } from '@/components/data-table'
  import { Button } from '@/components/ui/button'
  import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip'
  import { sleep } from '@/lib/utils'
  import { Trash2, UserX, UserCheck, Mail } from '@lucide/vue'
  import type { RowData } from '@tanstack/vue-table'
  import { computed, ref } from 'vue'
  import { toast } from 'vue-sonner'

  interface BulkActionsProps<T extends RowData> {
    table: DataTableInstance<T>
  }
  const { table } = defineProps<BulkActionsProps<T>>()

  const selectedRows = computed(() => table.getFilteredSelectedRowModel().rows)

  const handleBulkInvite = () => {
    const selectedUsers = selectedRows.value.map((row) => row.original as User)
    toast.promise(sleep(2000), {
      loading: 'Inviting users...',
      success: () => {
        table.resetRowSelection()
        return `Invited ${selectedUsers.length} user${selectedUsers.length > 1 ? 's' : ''}`
      },
      error: 'Error inviting users',
    })
    //table.resetRowSelection()
  }

  const handleBulkStatusChange = (status: 'active' | 'inactive') => {
    const selectedUsers = selectedRows.value.map((row) => row.original as User)
    toast.promise(sleep(2000), {
      loading: `${status === 'active' ? 'Activating' : 'Deactivating'} users...`,
      success: () => {
        table.resetRowSelection()
        return `${status === 'active' ? 'Activated' : 'Deactivated'} ${selectedUsers.length} user${selectedUsers.length > 1 ? 's' : ''}`
      },
      error: `Error ${status === 'active' ? 'activating' : 'deactivating'} users`,
    })
    //table.resetRowSelection()
  }

  const showDeleteDialog = ref(false)
  const onDelete = (event: Event) => {
    // Prevent focus returning to trigger element after closing
    ;(event.target as HTMLElement).blur()
    showDeleteDialog.value = true
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
          @click="handleBulkInvite"
          class="size-8"
          aria-label="Invite selected users"
        >
          <Mail />
          <span class="sr-only">Invite selected users</span>
        </Button>
      </TooltipTrigger>
      <TooltipContent>
        <p>Invite selected users</p>
      </TooltipContent>
    </Tooltip>

    <Tooltip>
      <TooltipTrigger as-child>
        <Button
          variant="outline"
          size="icon"
          @click="handleBulkStatusChange('active')"
          class="size-8"
          aria-label="Activate selected users"
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
      <TooltipTrigger as-child>
        <Button
          variant="outline"
          size="icon"
          @click="handleBulkStatusChange('inactive')"
          class="size-8"
          aria-label="Deactivate selected users"
        >
          <UserX />
          <span class="sr-only">Deactivate selected users</span>
        </Button>
      </TooltipTrigger>
      <TooltipContent>
        <p>Deactivate selected users</p>
      </TooltipContent>
    </Tooltip>

    <Tooltip>
      <TooltipTrigger as-child>
        <Button
          variant="destructive"
          size="icon"
          @click="onDelete"
          class="size-8"
          aria-label="Delete selected users"
        >
          <Trash2 />
          <span class="sr-only">Delete selected users</span>
        </Button>
      </TooltipTrigger>
      <TooltipContent>
        <p>Delete selected users</p>
      </TooltipContent>
    </Tooltip>

    <UserMultiDeleteDialog
      v-model="showDeleteDialog"
      :table="table"
    />
  </DataTableBulkActions>
</template>
