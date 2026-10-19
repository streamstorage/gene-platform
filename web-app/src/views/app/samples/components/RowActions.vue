<script setup lang="ts">
  import type { User } from '../data/schema'
  import UserActionDialog from './UserActionDialog.vue'
  import type { DataTableRow } from '@/components/data-table'
  import { Button } from '@/components/ui/button'
  import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuItem,
    DropdownMenuShortcut,
    DropdownMenuTrigger,
  } from '@/components/ui/dropdown-menu'
  import { EllipsisIcon, UserPen } from '@lucide/vue'
  import { ref } from 'vue'

  interface DataTableRowActionsProps {
    row: DataTableRow<User>
  }
  const props = defineProps<DataTableRowActionsProps>()

  const showEditDialog = ref(false)
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
        <DropdownMenuItem @click="showEditDialog = true">
          Edit
          <DropdownMenuShortcut>
            <UserPen :size="16" />
          </DropdownMenuShortcut>
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
    <UserActionDialog
      v-model="showEditDialog"
      :current-row="props.row.original"
    />
  </div>
</template>
