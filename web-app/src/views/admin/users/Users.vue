<script setup lang="ts">
  import { usersApi } from '@/api'
  import BulkActions from './components/BulkActions.vue'
  import { usersColumns } from './components/columns'
  import { roles, status } from './data/data'
  import { type User } from './data/schema'
  import Main from '@/components/Main.vue'
  import {
    type ColumnFilters,
    DataTable,
    DataTableBulkActions,
    DataTableToolbar,
    type DataTableToolbarFilter,
  } from '@/components/data-table'
  import { Button } from '@/components/ui/button'
  import { UserPlus } from '@lucide/vue'
  import { onMounted, ref } from 'vue'
  import { toast } from 'vue-sonner'

  const loading = ref(false)
  const data = ref<User[]>([])
  const columnFilters: ColumnFilters = [
    // per-column text filter
    { columnId: 'name', searchKey: 'name' },
    { columnId: 'active', searchKey: 'status', deserialize: (val: unknown) => val == 'true' },
    { columnId: 'role', searchKey: 'role', deserialize: (val: unknown) => Number(val) },
  ]
  const toolbarFilters = [
    { columnId: 'active', title: 'Status', options: status.map((s) => ({ ...s })) },
    { columnId: 'role', title: 'Role', options: roles.map((role) => ({ ...role })) },
  ] satisfies DataTableToolbarFilter[]

  const { listAll } = usersApi
  onMounted(async () => {
    loading.value = true
    try {
      data.value = await listAll()
    } catch (err) {
      toast.error(err instanceof Error ? `${err.name}: ${err.message}` : String(err))
    } finally {
      loading.value = false
    }
  })
</script>

<template>
  <Main class="flex flex-1 flex-col gap-4 sm:gap-6">
    <div class="flex flex-wrap items-end justify-between gap-2">
      <div>
        <h2 class="text-2xl font-bold tracking-tight">User List</h2>
        <p class="text-muted-foreground">Manage your users and their roles here.</p>
      </div>
      <div class="flex gap-2">
        <Button class="space-x-1"> <span>Add User</span> <UserPlus :size="18" /> </Button>
      </div>
    </div>
    <DataTable
      :data="data"
      :columns="usersColumns"
      :loading="loading"
      :column-filters="columnFilters"
    >
      <template #toolbar="{ table }">
        <DataTableToolbar
          :table="table"
          search-placeholder="Filter users..."
          search-key="name"
          :filters="toolbarFilters"
        />
      </template>
      <template #bulk-actions="{ table }">
        <DataTableBulkActions
          :table="table"
          entity-name="user"
        >
          <BulkActions :table="table" />
        </DataTableBulkActions>
      </template>
    </DataTable>
  </Main>
</template>
