<script setup lang="ts">
  import { Button } from '@/components/ui/button'
  import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuItem,
    DropdownMenuTrigger,
  } from '@/components/ui/dropdown-menu'
  import { cn } from '@/lib/utils'
  import { Check, Moon, Sun } from '@lucide/vue'
  import { useColorMode } from '@vueuse/core'

  const { store } = useColorMode()
</script>

<template>
  <DropdownMenu :modal="false">
    <DropdownMenuTrigger as-child>
      <Button
        variant="ghost"
        size="icon"
        class="scale-95 rounded-full"
      >
        <Sun class="size-[1.2rem] scale-100 rotate-0 transition-all dark:scale-0 dark:-rotate-90" />
        <Moon
          class="absolute size-[1.2rem] scale-0 rotate-90 transition-all dark:scale-100 dark:rotate-0"
        />
        <span class="sr-only">Toggle theme</span>
      </Button>
    </DropdownMenuTrigger>
    <DropdownMenuContent align="end">
      <DropdownMenuItem @click="store = 'light'">
        Light{{ ' ' }}
        <Check
          :size="14"
          :class="cn('ms-auto', store != 'light' && 'hidden')"
        />
      </DropdownMenuItem>
      <DropdownMenuItem @click="store = 'dark'">
        Dark
        <Check
          :size="14"
          :class="cn('ms-auto', store != 'dark' && 'hidden')"
        />
      </DropdownMenuItem>
      <DropdownMenuItem @click="store = 'auto'">
        System
        <Check
          :size="14"
          :class="cn('ms-auto', store != 'auto' && 'hidden')"
        />
      </DropdownMenuItem>
    </DropdownMenuContent>
  </DropdownMenu>
</template>
