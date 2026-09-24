<script setup lang="ts" generic="T extends RowData">
  import type { DataTableInstance } from '.'
  import { Button } from '@/components/ui/button'
  import {
    DropdownMenu,
    DropdownMenuCheckboxItem,
    DropdownMenuContent,
    DropdownMenuLabel,
    DropdownMenuSeparator,
    DropdownMenuTrigger,
  } from '@/components/ui/dropdown-menu'
  import { Settings2Icon } from '@lucide/vue'
  import type { RowData } from '@tanstack/vue-table'
  import { computed } from 'vue'

  interface DataTableViewOptionsProps {
    table: DataTableInstance<T>
  }

  const props = defineProps<DataTableViewOptionsProps>()

  const columns = computed(() =>
    props.table
      .getAllColumns()
      .filter((column) => typeof column.accessorFn !== 'undefined' && column.getCanHide())
  )
</script>

<template>
  <DropdownMenu>
    <DropdownMenuTrigger as-child>
      <Button
        variant="outline"
        size="sm"
        class="ms-auto hidden h-8 lg:flex"
      >
        <Settings2Icon class="size-4" />
        View
      </Button>
    </DropdownMenuTrigger>
    <DropdownMenuContent
      align="end"
      class="w-37.5"
    >
      <DropdownMenuLabel>Toggle columns</DropdownMenuLabel>
      <DropdownMenuSeparator />
      <DropdownMenuCheckboxItem
        v-for="column in columns"
        :key="column.id"
        class="capitalize"
        :model-value="column.getIsVisible()"
        @update:model-value="(value: boolean) => column.toggleVisibility(!!value)"
      >
        {{ column.id }}
      </DropdownMenuCheckboxItem>
    </DropdownMenuContent>
  </DropdownMenu>
</template>
