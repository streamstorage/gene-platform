<script setup lang="ts">
  import SignOutDialog from '@/components/layouts/SignOutDialog.vue'
  import { Avatar, AvatarFallback, AvatarImage } from '@/components/ui/avatar'
  import { Button } from '@/components/ui/button'
  import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuGroup,
    DropdownMenuItem,
    DropdownMenuLabel,
    DropdownMenuSeparator,
    DropdownMenuShortcut,
    DropdownMenuTrigger,
  } from '@/components/ui/dropdown-menu'
  import { useAuthStore } from '@/stores'
  import { storeToRefs } from 'pinia'
  import { ref } from 'vue'

  const authStore = useAuthStore()
  const { user } = storeToRefs(authStore)

  const showSignOutDialog = ref(false)
</script>

<template>
  <div>
    <DropdownMenu :modal="false">
      <DropdownMenuTrigger as-child>
        <Button
          variant="ghost"
          class="relative h-8 w-8 rounded-full"
        >
          <Avatar class="h-8 w-8">
            <AvatarImage
              :src="user.avatar"
              :alt="user.name"
            />
            <AvatarFallback>{{ user.initials }}</AvatarFallback>
          </Avatar>
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent
        class="w-56"
        align="end"
      >
        <DropdownMenuLabel class="font-normal">
          <div class="flex flex-col gap-1.5">
            <p class="text-sm leading-none font-medium">{{ user.name }}</p>
            <p class="text-xs leading-none text-muted-foreground">
              {{ user.email }}
            </p>
          </div>
        </DropdownMenuLabel>
        <DropdownMenuSeparator />
        <DropdownMenuGroup>
          <DropdownMenuItem as-child>
            <RouterLink to="/settings">
              Profile
              <DropdownMenuShortcut>⇧⌘P</DropdownMenuShortcut>
            </RouterLink>
          </DropdownMenuItem>
          <DropdownMenuItem asChild>
            <RouterLink to="/settings">
              Preference
              <DropdownMenuShortcut>⌘S</DropdownMenuShortcut>
            </RouterLink>
          </DropdownMenuItem>
        </DropdownMenuGroup>
        <DropdownMenuSeparator />
        <DropdownMenuItem
          variant="destructive"
          @click="showSignOutDialog = true"
        >
          Sign out
          <DropdownMenuShortcut class="text-current"> ⇧⌘Q </DropdownMenuShortcut>
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
    <SignOutDialog
      v-if="showSignOutDialog"
      v-model="showSignOutDialog"
    />
  </div>
</template>
