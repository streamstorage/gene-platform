<script setup lang="ts" generic="T extends RowData">
  import type { DataTableColumn } from '.'
  import { Button } from '@/components/ui/button'
  import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuItem,
    DropdownMenuSeparator,
    DropdownMenuTrigger,
  } from '@/components/ui/dropdown-menu'
  import { cn } from '@/lib/utils'
  import { ArrowDownIcon, ArrowUpIcon, ChevronsUpDownIcon, EyeOffIcon } from '@lucide/vue'
  import type { RowData } from '@tanstack/vue-table'
  import { type HTMLAttributes } from 'vue'

  interface DataTableColumnHeaderProps {
    column: DataTableColumn<T>
    title: string
    class?: HTMLAttributes['class']
  }
  const props = defineProps<DataTableColumnHeaderProps>()
</script>

<template>
  <div
    v-if="!props.column.getCanSort()"
    :class="cn(props.class)"
  >
    {{ props.title }}
  </div>
  <div
    v-else
    :class="cn('flex items-center space-x-2', props.class)"
  >
    <DropdownMenu>
      <DropdownMenuTrigger as-child>
        <Button
          variant="ghost"
          size="sm"
          class="h-8 data-[state=open]:bg-accent"
        >
          <span>{{ props.title }}</span>
          <ArrowDownIcon
            v-if="props.column.getIsSorted() === 'desc'"
            class="ms-2 h-4 w-4"
          />
          <ArrowUpIcon
            v-else-if="props.column.getIsSorted() === 'asc'"
            class="ms-2 h-4 w-4"
          />
          <ChevronsUpDownIcon
            v-else
            class="ms-2 h-4 w-4"
          />
        </Button>
      </DropdownMenuTrigger>

      <DropdownMenuContent align="start">
        <DropdownMenuItem @click="props.column.toggleSorting(false)">
          <ArrowUpIcon class="size-3.5 text-muted-foreground/70" />
          Asc
        </DropdownMenuItem>
        <DropdownMenuItem @click="props.column.toggleSorting(true)">
          <ArrowDownIcon class="size-3.5 text-muted-foreground/70" />
          Desc
        </DropdownMenuItem>
        <DropdownMenuItem @click="props.column.clearSorting()">
          <ChevronsUpDownIcon class="size-3.5 text-muted-foreground/70" />
          Clear Sorting
        </DropdownMenuItem>
        <template v-if="props.column.getCanHide()">
          <DropdownMenuSeparator />
          <DropdownMenuItem @click="props.column.toggleVisibility(false)">
            <EyeOffIcon class="size-3.5 text-muted-foreground/70" />
            Hide
          </DropdownMenuItem>
        </template>
      </DropdownMenuContent>
    </DropdownMenu>
  </div>
</template>
