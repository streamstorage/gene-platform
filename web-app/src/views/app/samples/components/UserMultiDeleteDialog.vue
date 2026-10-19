<script setup lang="ts" generic="T extends RowData">
  import ConfirmDialog from '@/components/ConfirmDialog.vue'
  import type { DataTableInstance } from '@/components/data-table'
  import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert'
  import { Input } from '@/components/ui/input'
  import { Label } from '@/components/ui/label'
  import { sleep } from '@/lib/utils'
  import { AlertTriangle } from '@lucide/vue'
  import type { RowData } from '@tanstack/vue-table'
  import { ref } from 'vue'
  import { toast } from 'vue-sonner'

  const open = defineModel<boolean>()

  type UserMultiDeleteDialogProps = {
    table: DataTableInstance<T>
  }

  const props = defineProps<UserMultiDeleteDialogProps>()
  const CONFIRM_WORD = 'DELETE'
  const value = ref('')

  const selectedRows = props.table.getFilteredSelectedRowModel().rows

  const handleDelete = async () => {
    if (value.value.trim() !== CONFIRM_WORD) {
      toast.error(`Please type "${CONFIRM_WORD}" to confirm.`)
      return
    }

    open.value = false

    toast.promise(sleep(2000), {
      loading: 'Deleting users...',
      success: () => {
        props.table.resetRowSelection()
        return `Deleted ${selectedRows.length} ${selectedRows.length > 1 ? 'users' : 'user'}`
      },
      error: 'Error',
    })
  }
</script>

<template>
  <ConfirmDialog
    v-model="open"
    form="user-multi-delete-form"
    :disabled="value.trim() !== CONFIRM_WORD"
    confirmText="Delete"
    destructive
  >
    <template #title>
      <span class="text-destructive">
        <AlertTriangle
          class="me-1 inline-block stroke-destructive"
          :size="18"
        />{{ ' ' }} Delete {{ selectedRows.length }}{{ ' ' }}
        {{ selectedRows.length > 1 ? 'users' : 'user' }}
      </span>
    </template>

    <template #description>
      <div>
        <form
          class="space-y-4"
          id="user-multi-delete-form"
          @submit="
            (e) => {
              e.preventDefault()
              handleDelete()
            }
          "
        >
          <p class="mb-2">
            Are you sure you want to delete the selected users? <br />
            This action cannot be undone.
          </p>

          <Label class="my-4 flex flex-col items-start gap-1.5">
            <span class="">Confirm by typing "{{ CONFIRM_WORD }}":</span>
            <Input
              v-model="value"
              :placeholder="`Type &quot;${CONFIRM_WORD}&quot; to confirm.`"
              auto-focus
            />
          </Label>

          <Alert variant="destructive">
            <AlertTitle>Warning!</AlertTitle>
            <AlertDescription>
              Please be careful, this operation can not be rolled back.
            </AlertDescription>
          </Alert>
        </form>
      </div>
    </template>
  </ConfirmDialog>
</template>
