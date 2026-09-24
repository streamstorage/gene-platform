<script setup lang="ts">
  import AppSidebarNavGroup from './AppSidebarNavGroup.vue'
  import AppSidebarNavUser from './AppSidebarNavUser.vue'
  import AppSidebarTitle from './AppSidebarTitle.vue'
  import {
    Sidebar,
    SidebarContent,
    SidebarFooter,
    SidebarHeader,
    SidebarRail,
  } from '@/components/ui/sidebar'
  import { navGroups, navAdminGroups } from '@/constants/sidebar'
  import { computed } from 'vue'

  const props = defineProps({
    isAdmin: { type: Boolean, default: false },
  })

  const groups = computed(() => (props.isAdmin ? navAdminGroups : navGroups))
</script>

<template>
  <Sidebar
    collapsible="icon"
    variant="inset"
  >
    <SidebarHeader>
      <AppSidebarTitle />
    </SidebarHeader>
    <SidebarContent>
      <AppSidebarNavGroup
        v-for="group in groups"
        :key="group.title"
        :group="group"
      />
    </SidebarContent>
    <SidebarFooter>
      <AppSidebarNavUser />
    </SidebarFooter>
    <SidebarRail />
  </Sidebar>
</template>
