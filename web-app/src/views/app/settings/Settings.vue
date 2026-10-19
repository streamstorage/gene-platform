<script setup lang="ts">
  import SidebarNav from './components/SidebarNav.vue'
  import MainPage from '@/components/layouts/MainPage.vue'
  import { Separator } from '@/components/ui/separator'
  import { Palette, UserCog } from '@lucide/vue'

  const sidebarNavItems = [
    {
      title: 'Profile',
      href: '/app/settings/profile',
      icon: UserCog,
    },
    {
      title: 'Preference',
      href: '/app/settings/preference',
      icon: Palette,
    },
  ]
</script>

<template>
  <MainPage fixed>
    <div class="space-y-0.5">
      <h1 class="text-2xl font-bold tracking-tight md:text-3xl">Settings</h1>
      <p class="text-muted-foreground">Manage your account settings and set preferences.</p>
    </div>
    <Separator class="my-4 lg:my-6" />
    <div
      class="flex flex-1 flex-col space-y-2 overflow-hidden md:space-y-2 lg:flex-row lg:space-y-0 lg:space-x-12"
    >
      <aside class="top-0 lg:sticky lg:w-1/5">
        <SidebarNav :items="sidebarNavItems" />
      </aside>
      <div class="flex w-full overflow-y-hidden p-1">
        <router-view />
      </div>
    </div>
  </MainPage>
</template>
