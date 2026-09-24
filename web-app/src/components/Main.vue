<script setup lang="ts">
  import { cn } from '@/lib/utils'
  import { type HTMLAttributes } from 'vue'
  const props = defineProps<{
    fixed?: boolean
    fluid?: boolean
    class?: HTMLAttributes['class']
  }>()
</script>

<template>
  <main
    :data-layout="props.fixed ? 'fixed' : 'auto'"
    :class="
      cn(
        'px-4 py-6',

        // If layout is fixed, make the main container flex and grow
        fixed && 'flex grow flex-col overflow-hidden',

        // If layout is not fluid, set the max-width
        !fluid && '@7xl/content:mx-auto @7xl/content:w-full @7xl/content:max-w-7xl',
        props.class
      )
    "
  >
    <slot></slot>
  </main>
</template>
