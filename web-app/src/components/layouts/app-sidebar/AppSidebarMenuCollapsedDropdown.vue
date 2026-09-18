<script setup lang="ts">
  import { checkIsActive, type NavCollapsible } from '.'
  import AppSidebarBadge from './AppSidebarBadge.vue'
  import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuItem,
    DropdownMenuLabel,
    DropdownMenuSeparator,
    DropdownMenuTrigger,
  } from '@/components/ui/dropdown-menu'
  import { SidebarMenuItem, SidebarMenuButton } from '@/components/ui/sidebar'
  import { ChevronRight } from '@lucide/vue'
  import { toRef } from 'vue'

  const props = defineProps<{
    href: string
    item: NavCollapsible
  }>()

  const item = toRef(() => props.item)
</script>

<template>
  <SidebarMenuItem>
    <DropdownMenu>
      <DropdownMenuTrigger as-child>
        <SidebarMenuButton
          :tooltip="item.title"
          :is-active="checkIsActive(props.href, item)"
        >
          <component
            v-if="item.icon"
            :is="item.icon"
          />
          <span>{{ item.title }}</span>
          <AppSidebarBadge v-if="item.badge">{{ item.badge }}</AppSidebarBadge>
          <ChevronRight
            class="ms-auto transition-transform duration-200 group-data-[state=open]/collapsible:rotate-90"
          />
        </SidebarMenuButton>
      </DropdownMenuTrigger>
      <DropdownMenuContent
        side="right"
        align="start"
        :side-offset="4"
      >
        <DropdownMenuLabel> {{ item.title }} {{ item.badge ?? '' }} </DropdownMenuLabel>
        <DropdownMenuSeparator />
        <DropdownMenuItem
          v-for="sub in item.items"
          :key="`${sub.title}-${sub.url}`"
          as-child
        >
          <RouterLink
            :to="sub.url"
            :class="checkIsActive(href, sub) ? 'bg-secondary' : ''"
          >
            <component
              v-if="sub.icon"
              :is="sub.icon"
            />
            <span class="max-w-52 text-wrap">{{ sub.title }}</span>
            <span
              v-if="sub.badge"
              class="ms-auto text-xs"
              >{{ sub.badge }}</span
            >
          </RouterLink>
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  </SidebarMenuItem>
</template>
