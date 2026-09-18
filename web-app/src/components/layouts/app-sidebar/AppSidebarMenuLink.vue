<script setup lang="ts">
  import { checkIsActive, type NavLink } from '.'
  import AppSidebarBadge from './AppSidebarBadge.vue'
  import { SidebarMenuItem, SidebarMenuButton, useSidebar } from '@/components/ui/sidebar'
  import { toRef } from 'vue'

  const props = defineProps<{
    href: string
    item: NavLink
  }>()

  const item = toRef(() => props.item)

  const { setOpenMobile } = useSidebar()
</script>

<template>
  <SidebarMenuItem>
    <SidebarMenuButton
      as-child
      :is-active="checkIsActive(props.href, item)"
      :tooltip="item.title"
    >
      <RouterLink
        :to="item.url"
        @click="() => setOpenMobile(false)"
      >
        <component
          v-if="item.icon"
          :is="item.icon"
        />
        <span>{{ item.title }}</span>
        <AppSidebarBadge v-if="item.badge">{{ item.badge }}</AppSidebarBadge>
      </RouterLink>
    </SidebarMenuButton>
  </SidebarMenuItem>
</template>
