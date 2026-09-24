<script setup lang="ts">
  import BulkActions from './components/BulkActions.vue'
  import { usersColumns } from './components/columns'
  import { roles, status } from './data/data'
  import { users } from './data/users'
  import Main from '@/components/Main.vue'
  import { DataTable, DataTableBulkActions, DataTableToolbar } from '@/components/data-table'
  import { Button } from '@/components/ui/button'
  import { MailPlus, UserPlus } from '@lucide/vue'
  import { onMounted, ref } from 'vue'
  import { sleep } from '@/lib/utils'

  const loading = ref(false)

  onMounted(async () => {
    loading.value = true
    await sleep(1000)
    loading.value = false
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
        <Button
          variant="outline"
          class="space-x-1"
        >
          <span>Invite User</span> <MailPlus :size="18" />
        </Button>
        <Button class="space-x-1"> <span>Add User</span> <UserPlus :size="18" /> </Button>
      </div>
    </div>
    <DataTable
      :data="users"
      :columns="usersColumns"
      :loading="loading"
    >
      <template #toolbar="{ table }">
        <DataTableToolbar
          :table="table"
          search-placeholder="Filter users..."
          search-key="name"
          :filters="[
            {
              columnId: 'status',
              title: 'Status',
              options: status.map((s) => ({ ...s })),
            },
            {
              columnId: 'role',
              title: 'Role',
              options: roles.map((role) => ({ ...role })),
            },
          ]"
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
