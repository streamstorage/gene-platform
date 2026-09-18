<script setup lang="ts">
  import { Button } from '@/components/ui/button'
  import {
    SidebarMenu,
    SidebarMenuButton,
    SidebarMenuItem,
    useSidebar,
  } from '@/components/ui/sidebar'
  import { Dna, Menu, X } from '@lucide/vue'

  const { setOpenMobile } = useSidebar()
</script>

<template>
  <SidebarMenu>
    <SidebarMenuItem>
      <SidebarMenuButton
        size="lg"
        class="data-[state=open]:bg-sidebar-accent data-[state=open]:text-sidebar-accent-foreground"
      >
        <div
          class="flex aspect-square size-8 items-center justify-center rounded-lg bg-sidebar-primary text-sidebar-primary-foreground"
        >
          <Dna class="size-4" />
        </div>
        <RouterLink
          to="/"
          @click="setOpenMobile(false)"
          class="grid flex-1 text-start text-sm leading-tight"
        >
          <span class="truncate font-semibold">Gene Platform</span>
          <span class="truncate text-xs">Welcome</span>
        </RouterLink>

        <Button
          data-sidebar="trigger"
          data-slot="sidebar-trigger"
          variant="ghost"
          size="icon"
          class="aspect-square size-8 max-md:scale-125"
        >
          <X class="md:hidden" />
          <Menu class="max-md:hidden" />
          <span class="sr-only"></span>
        </Button>
      </SidebarMenuButton>
    </SidebarMenuItem>
  </SidebarMenu>
</template>
