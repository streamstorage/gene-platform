<script setup lang="ts">
  import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuItem,
    DropdownMenuLabel,
    DropdownMenuSeparator,
    DropdownMenuShortcut,
    DropdownMenuTrigger,
  } from '@/components/ui/dropdown-menu'
  import {
    SidebarMenu,
    SidebarMenuButton,
    SidebarMenuItem,
    useSidebar,
  } from '@/components/ui/sidebar'
  import { useAuthStore } from '@/stores'
  import { ChevronsUpDown, CircleAlert, Command, Dna } from '@lucide/vue'
  import { storeToRefs } from 'pinia'
  import { computed } from 'vue'
  import { useRoute, useRouter } from 'vue-router'

  const { isMobile } = useSidebar()
  const { push } = useRouter()

  const route = useRoute()
  const activeApp = computed(() => {
    switch (route.meta.guard) {
      case 'user':
        return {
          logo: Dna,
          name: 'Gene Platform',
          sub: 'Welcome',
        }
      case 'admin':
        return {
          logo: Command,
          name: 'System Admin',
          sub: 'App Management',
        }
      default:
        return {
          logo: CircleAlert,
          name: 'Unknown',
          sub: 'Error',
        }
    }
  })

  const authStore = useAuthStore()
  const { adminAuthorized } = storeToRefs(authStore)
</script>

<template>
  <SidebarMenu>
    <SidebarMenuItem>
      <DropdownMenu :modal="false">
        <DropdownMenuTrigger as-child>
          <SidebarMenuButton
            size="lg"
            class="data-[state=open]:bg-sidebar-accent data-[state=open]:text-sidebar-accent-foreground"
          >
            <div
              class="flex aspect-square size-8 items-center justify-center rounded-lg bg-sidebar-primary text-sidebar-primary-foreground"
            >
              <component
                :is="activeApp.logo"
                class="size-4"
              />
            </div>
            <div class="grid flex-1 text-start text-sm leading-tight">
              <span class="truncate font-semibold">
                {{ activeApp.name }}
              </span>
              <span class="truncate text-xs">{{ activeApp.sub }}</span>
            </div>
            <ChevronsUpDown class="ms-auto" />
          </SidebarMenuButton>
        </DropdownMenuTrigger>
        <DropdownMenuContent
          class="w-(--reka-dropdown-menu-trigger-width) min-w-56 rounded-lg"
          align="start"
          :side="isMobile ? 'bottom' : 'right'"
          :side-offset="4"
        >
          <DropdownMenuLabel class="text-xs text-muted-foreground"> Apps </DropdownMenuLabel>
          <DropdownMenuItem
            @click="push('/app')"
            class="gap-2 p-2"
          >
            <div class="flex size-6 items-center justify-center rounded-sm border">
              <Dna class="size-4 shrink-0" />
            </div>
            Gene Platform
            <DropdownMenuShortcut>⌘1</DropdownMenuShortcut>
          </DropdownMenuItem>
          <DropdownMenuSeparator v-if="adminAuthorized" />
          <DropdownMenuItem
            v-if="adminAuthorized"
            @click="push('/admin')"
            class="gap-2 p-2"
          >
            <div class="flex size-6 items-center justify-center rounded-sm border">
              <Command class="size-4 shrink-0" />
            </div>
            System Admin
          </DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>
    </SidebarMenuItem>
  </SidebarMenu>
</template>
