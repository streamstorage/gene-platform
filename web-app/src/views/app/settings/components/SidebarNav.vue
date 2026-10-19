<script setup lang="ts">
  import { buttonVariants } from '@/components/ui/button'
  import { ScrollArea } from '@/components/ui/scroll-area'
  import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
  } from '@/components/ui/select'
  import { cn } from '@/lib/utils'
  import { type Component, type HTMLAttributes } from 'vue'
  import { useRoute } from 'vue-router'

  // Prevents root div from auto-inheriting all fallthrough attributes
  defineOptions({
    inheritAttrs: false,
  })

  const props = defineProps<{
    items: {
      href: string
      title: string
      icon: Component
    }[]
    class?: HTMLAttributes['class']
  }>()

  const route = useRoute()
</script>

<template>
  <div class="p-1 md:hidden">
    <Select
      value="{val}"
      onValueChange="{handleSelect}"
    >
      <SelectTrigger class="h-12 sm:w-48">
        <SelectValue placeholder="Theme" />
      </SelectTrigger>
      <SelectContent>
        <SelectItem
          v-for="item in props.items"
          :key="item.href"
          value="item.href"
        >
          <div class="flex gap-x-4 px-2 py-1">
            <span class="scale-125">
              <component
                :is="item.icon"
                :size="18"
              />
            </span>
            <span class="text-md">{{ item.title }}</span>
          </div>
        </SelectItem>
      </SelectContent>
    </Select>
  </div>

  <ScrollArea
    orientation="horizontal"
    type="auto"
    class="hidden w-full min-w-40 bg-background px-1 py-2 md:block"
  >
    <nav
      :class="cn('flex space-x-2 py-1 lg:flex-col lg:space-y-1 lg:space-x-0', props.class)"
      v-bind="$attrs"
    >
      <RouterLink
        v-for="item in props.items"
        :key="item.href"
        :to="item.href"
        :class="
          cn(
            buttonVariants({ variant: 'ghost' }),
            route.path === item.href
              ? 'bg-muted hover:bg-accent'
              : 'hover:bg-accent hover:underline',
            'justify-start'
          )
        "
      >
        <span class="me-2"
          ><component
            :is="item.icon"
            :size="18"
        /></span>
        {{ item.title }}
      </RouterLink>
    </nav>
  </ScrollArea>
</template>
