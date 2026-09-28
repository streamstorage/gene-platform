<script setup lang="ts" generic="T extends RowData">
  import type { DataTableInstance } from '.'
  import { Badge } from '@/components/ui/badge'
  import { Button } from '@/components/ui/button'
  import { Separator } from '@/components/ui/separator'
  import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip'
  import { cn } from '@/lib/utils'
  import { X } from '@lucide/vue'
  import type { RowData } from '@tanstack/vue-table'
  import { computed, ref } from 'vue'

  interface BulkActionsProps<T extends RowData> {
    table: DataTableInstance<T>
    entityName: string
  }

  const { table, entityName } = defineProps<BulkActionsProps<T>>()

  const announcement = ref('')

  const selectedRows = computed(() => table.getSelectedRowModel().rows)
  const selectedCount = computed(() => selectedRows.value.length)

  function handleClearSelection() {
    table.resetRowSelection()
  }
</script>

<template>
  <template v-if="selectedCount > 0">
    <div
      aria-live="polite"
      aria-atomic="true"
      class="sr-only"
      role="status"
    >
      {{ announcement }}
    </div>

    <div
      role="toolbar"
      :aria-label="`Bulk actions for ${selectedCount} selected ${entityName}${selectedCount > 1 ? 's' : ''}`"
      aria-describedby="bulk-actions-description"
      :tab-index="-1"
      :class="
        cn(
          'fixed bottom-6 left-1/2 z-50 -translate-x-1/2 rounded-xl',
          'transition-all delay-100 duration-300 ease-out hover:scale-105',
          'focus-visible:ring-2 focus-visible:ring-ring/50 focus-visible:outline-none'
        )
      "
    >
      <div
        :class="
          cn(
            'p-2 shadow-xl',
            'rounded-xl border',
            'bg-background/95 backdrop-blur-lg supports-backdrop-filter:bg-background/60',
            'flex items-center gap-x-2'
          )
        "
      >
        <Tooltip>
          <TooltipTrigger as-child>
            <Button
              variant="outline"
              size="icon"
              @click="handleClearSelection"
              class="size-6 rounded-full"
              aria-label="Clear selection"
            >
              <X />
              <span class="sr-only">Clear selection</span>
            </Button>
          </TooltipTrigger>
          <TooltipContent>
            <p>Clear selection</p>
          </TooltipContent>
        </Tooltip>

        <Separator
          class="h-5"
          orientation="vertical"
          aria-hidden="true"
        />

        <div
          className="flex items-center gap-x-1 text-sm"
          id="bulk-actions-description"
        >
          <Badge
            variant="default"
            class="min-w-8 rounded-lg"
            :aria-label="`${selectedCount} selected`"
          >
            {{ selectedCount }} </Badge
          >{{ ' ' }}
          <span class="hidden sm:inline">
            {{ entityName }}
            {{ selectedCount > 1 ? 's' : '' }} </span
          >{{ ' ' }}
          selected
        </div>

        <Separator
          class="h-5"
          orientation="vertical"
          aria-hidden="true"
        />

        <slot />
      </div>
    </div>
  </template>
</template>
