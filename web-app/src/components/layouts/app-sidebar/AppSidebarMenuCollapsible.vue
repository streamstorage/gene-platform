<script setup lang="ts">
  import { checkIsActive, type NavCollapsible } from '.'
  import AppSidebarBadge from './AppSidebarBadge.vue'
  import { Collapsible, CollapsibleContent, CollapsibleTrigger } from '@/components/ui/collapsible'
  import {
    SidebarMenuButton,
    SidebarMenuItem,
    SidebarMenuSub,
    SidebarMenuSubButton,
    SidebarMenuSubItem,
    useSidebar,
  } from '@/components/ui/sidebar'
  import { ChevronRight } from '@lucide/vue'
  import { toRef } from 'vue'

  const props = defineProps<{
    href: string
    item: NavCollapsible
  }>()

  const item = toRef(() => props.item)

  const { setOpenMobile } = useSidebar()
</script>

<template>
  <Collapsible
    asChild
    :default-open="checkIsActive(href, item, true)"
    class="group/collapsible"
  >
    <SidebarMenuItem>
      <CollapsibleTrigger as-child>
        <SidebarMenuButton :tooltip="item.title">
          <component
            v-if="item.icon"
            :is="item.icon"
          />
          <span>{{ item.title }}</span>
          <AppSidebarBadge v-if="item.badge">{{ item.badge }}</AppSidebarBadge>
          <ChevronRight
            class="ms-auto transition-transform duration-200 group-data-[state=open]/collapsible:rotate-90 rtl:rotate-180"
          />
        </SidebarMenuButton>
      </CollapsibleTrigger>
      <CollapsibleContent class="CollapsibleContent">
        <SidebarMenuSub>
          <SidebarMenuSubItem
            v-for="subItem in item.items"
            :key="subItem.title"
          >
            <SidebarMenuSubButton
              as-child
              :is-active="checkIsActive(href, subItem)"
            >
              <RouterLink
                :to="subItem.url"
                @click="setOpenMobile(false)"
              >
                <component
                  v-if="subItem.icon"
                  :is="subItem.icon"
                />
                <span>{{ subItem.title }}</span>
                <AppSidebarBadge v-if="subItem.badge">{{ subItem.badge }}</AppSidebarBadge>
              </RouterLink>
            </SidebarMenuSubButton>
          </SidebarMenuSubItem>
        </SidebarMenuSub>
      </CollapsibleContent>
    </SidebarMenuItem>
  </Collapsible>
</template>
