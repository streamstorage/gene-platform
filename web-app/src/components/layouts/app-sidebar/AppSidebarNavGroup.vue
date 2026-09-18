<script setup lang="ts">
  import { SidebarGroup, SidebarGroupLabel, SidebarMenu, useSidebar } from '@/components/ui/sidebar'
  import { type NavGroup } from '.'
  import AppSidebarMenuCollapsedDropdown from './AppSidebarMenuCollapsedDropdown.vue'
  import AppSidebarMenuCollapsible from './AppSidebarMenuCollapsible.vue'
  import AppSidebarMenuLink from './AppSidebarMenuLink.vue'
  import { computed, toRef } from 'vue'
  import { useRoute } from 'vue-router'

  const props = defineProps<{
    group: NavGroup
  }>()

  const group = toRef(() => props.group)

  const route = useRoute()
  const href = computed(() => route.path)

  const { state, isMobile } = useSidebar()
</script>

<template>
  <SidebarGroup>
    <SidebarGroupLabel>{{ group.title }}</SidebarGroupLabel>
    <SidebarMenu>
      <template
        v-for="item in group.items"
        :key="`${item.title}-${item.url}`"
      >
        <AppSidebarMenuLink
          v-if="!item.items"
          :item="item"
          :href="href"
        />
        <AppSidebarMenuCollapsedDropdown
          v-else-if="state === 'collapsed' && !isMobile"
          :item="item"
          :href="href"
        />
        <AppSidebarMenuCollapsible
          v-else
          :item="item"
          :href="href"
        />
      </template>
    </SidebarMenu>
  </SidebarGroup>
</template>
