<script setup lang="ts">
  import { computed, type HTMLAttributes } from 'vue'
  import { cn } from '@/lib/utils'

  const props = defineProps<{
    items: { name: string; value: number }[]
    valueFormatter: (n: number) => string
    class?: HTMLAttributes['class']
  }>()

  const max = computed(() => Math.max(...props.items.map((i) => i.value), 1))
</script>

<template>
  <ul class="space-y-3">
    <li
      v-for="i in props.items"
      :key="i.name"
      class="flex items-center justify-between gap-3"
    >
      <div class="min-w-0 flex-1">
        <div class="mb-1 truncate text-xs text-muted-foreground">
          {{ i.name }}
        </div>
        <div class="h-2.5 w-full rounded-full bg-muted">
          <div
            :class="cn('h-2.5 rounded-full', props.class)"
            :style="`width: ${Math.round((i.value / max) * 100)}%`"
          />
        </div>
      </div>
      <div class="ps-2 text-xs font-medium tabular-nums">
        {{ valueFormatter(i.value) }}
      </div>
    </li>
  </ul>
</template>
