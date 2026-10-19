<script setup lang="ts">
  import { cn } from '@/lib/utils'
  import { Separator } from '@/components/ui/separator'
  import { SidebarTrigger } from '@/components/ui/sidebar'
  import { type HTMLAttributes, onBeforeUnmount, onMounted, ref } from 'vue'

  const props = defineProps<{
    fixed?: boolean
    class?: HTMLAttributes['class']
  }>()

  const offset = ref(0)

  const handleScroll = () => {
    offset.value = window.scrollY || window.pageYOffset
  }

  onMounted(() => {
    window.addEventListener('scroll', handleScroll)
  })

  onBeforeUnmount(() => {
    window.removeEventListener('scroll', handleScroll)
  })
</script>

<template>
  <header
    :class="
      cn(
        'z-50 h-16',
        fixed && 'header-fixed peer/header sticky top-0 w-[inherit]',
        offset > 10 && fixed ? 'shadow' : 'shadow-none',
        props.class
      )
    "
  >
    <div
      :class="
        cn(
          'relative flex h-full items-center gap-3 p-4 sm:gap-4',
          offset > 10 &&
            fixed &&
            'after:absolute after:inset-0 after:-z-10 after:bg-background/20 after:backdrop-blur-lg'
        )
      "
    >
      <SidebarTrigger
        variant="outline"
        class="max-md:scale-125"
      />
      <Separator
        orientation="vertical"
        class="h-6"
      />
      <slot></slot>
    </div>
  </header>
</template>
