<script setup lang="ts">
  import BulkActions from './components/BulkActions.vue'
  import UserActionDialog from './components/UserActionDialog.vue'
  import UserInviteDialog from './components/UserInviteDialog.vue'
  import { usersColumns } from './components/columns'
  import { roles, status } from './data/data'
  import { type User } from './data/schema'
  import { users } from './data/users'
  import {
    type ColumnFilters,
    DataTable,
    DataTableToolbar,
    type DataTableToolbarFilter,
  } from '@/components/data-table'
  import MainPage from '@/components/layouts/MainPage.vue'
  import { Button } from '@/components/ui/button'
  import { MailPlus, UserPlus } from '@lucide/vue'
  import { onMounted, ref } from 'vue'
  import { sleep } from '@/lib/utils'

  const loading = ref(false)
  const data = ref<User[]>([])
  const columnFilaters: ColumnFilters = [
    // per-column text filter
    { columnId: 'name', searchKey: 'name' },
    { columnId: 'status', searchKey: 'status' },
    { columnId: 'role', searchKey: 'role' },
  ]
  const toolbarFilters = [
    { columnId: 'status', title: 'Status', options: status.map((s) => ({ ...s })) },
    { columnId: 'role', title: 'Role', options: roles.map((role) => ({ ...role })) },
  ] satisfies DataTableToolbarFilter[]

  const showInviteDialog = ref(false)
  const showAddDialog = ref(false)

  onMounted(async () => {
    loading.value = true
    await sleep(1000)
    data.value = users
    loading.value = false
  })
</script>

<template>
  <MainPage class="flex flex-1 flex-col gap-4 sm:gap-6">
    <div class="flex flex-wrap items-end justify-between gap-2">
      <div>
        <h2 class="text-2xl font-bold tracking-tight">User List</h2>
        <p class="text-muted-foreground">Manage your users and their roles here.</p>
      </div>
      <div class="flex gap-2">
        <Button
          variant="outline"
          class="space-x-1"
          @click="showInviteDialog = true"
        >
          <span>Invite User</span> <MailPlus :size="18" />
        </Button>
        <Button
          class="space-x-1"
          @click="showAddDialog = true"
        >
          <span>Add User</span> <UserPlus :size="18" />
        </Button>
        <UserInviteDialog v-model="showInviteDialog" />
        <UserActionDialog v-model="showAddDialog" />
      </div>
    </div>
    <DataTable
      :data="data"
      :columns="usersColumns"
      :loading="loading"
      :column-filters="columnFilaters"
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
        <BulkActions :table="table" />
      </template>
    </DataTable>
  </MainPage>
</template>
