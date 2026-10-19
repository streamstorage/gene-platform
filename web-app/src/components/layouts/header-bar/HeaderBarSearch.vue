<script setup lang="ts">
  import CommandMenu from '../CommandMenu.vue'
  import { Button } from '@/components/ui/button'
  import { cn } from '@/lib/utils'
  import { SearchIcon } from '@lucide/vue'
  import { useEventListener } from '@vueuse/core'
  import { type HTMLAttributes, ref } from 'vue'

  const props = withDefaults(
    defineProps<{
      placeholder?: string
      class?: HTMLAttributes['class']
    }>(),
    {
      placeholder: 'Search',
    }
  )

  const open = ref(false)

  useEventListener('keydown', (event: KeyboardEvent) => {
    if (event.key === 'k' && (event.metaKey || event.ctrlKey)) {
      event.preventDefault()
      open.value = !open.value
    }
  })
</script>

<template>
  <Button
    variant="outline"
    :class="
      cn(
        'group relative h-8 w-full flex-1 justify-start rounded-md bg-muted/25 text-sm font-normal text-muted-foreground shadow-none hover:bg-accent sm:w-40 sm:pe-12 md:flex-none lg:w-52 xl:w-64',
        props.class
      )
    "
    aria-keyshortcuts="Meta+K Control+K"
    @click="open = true"
  >
    <SearchIcon
      aria-hidden="true"
      class="absolute inset-s-1.5 top-1/2 -translate-y-1/2"
      :size="16"
    />
    <span class="ms-4">{{ props.placeholder }}</span>
    <kbd
      class="pointer-events-none absolute inset-e-[0.3rem] top-[0.3rem] hidden h-5 items-center gap-1 rounded border bg-muted px-1.5 font-mono text-[10px] font-medium opacity-100 select-none group-hover:bg-accent sm:flex"
    >
      <span class="text-xs">⌘</span>K
    </kbd>
  </Button>

  <CommandMenu v-model="open" />
</template>
