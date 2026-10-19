<script setup lang="ts">
  import {
    CommandDialog,
    CommandEmpty,
    CommandGroup,
    CommandInput,
    CommandItem,
    CommandList,
    CommandSeparator,
  } from '@/components/ui/command'
  import { ScrollArea } from '@/components/ui/scroll-area'
  import { navGroups } from '@/constants/sidebar'
  import { ArrowRight, ChevronRight, Laptop, Moon, Sun } from '@lucide/vue'
  import { useColorMode } from '@vueuse/core'
  import { useRouter } from 'vue-router'

  const open = defineModel<boolean>()

  const { store } = useColorMode()

  const { push } = useRouter()
  const runCommand = (f: () => void) => {
    open.value = false
    f()
  }
</script>

<template>
  <CommandDialog
    :modal="true"
    v-model:open="open"
  >
    <CommandInput placeholder="Type a command or search..." />
    <CommandList>
      <ScrollArea
        type="hover"
        class="h-72 pe-1"
      >
        <CommandEmpty>No results found.</CommandEmpty>
        <CommandGroup
          v-for="group in navGroups"
          :key="group.title"
          :heading="group.title"
        >
          <template
            v-for="(navItem, i) in group.items"
            :key="`${navItem.title}-${i}`"
          >
            <CommandItem
              v-if="navItem.url"
              :value="navItem.title"
              @select="runCommand(() => push(navItem.url))"
            >
              <div class="flex size-4 items-center justify-center">
                <ArrowRight class="text-muted-foreground/80" />
              </div>
              {{ navItem.title }}
            </CommandItem>
            <template v-else>
              <CommandItem
                v-for="(subItem, j) in navItem.items"
                :key="`${navItem.title}-${subItem.url}-${j}`"
                :value="`${navItem.title}-${subItem.url}`"
                @select="runCommand(() => push(subItem.url))"
              >
                <div class="flex size-4 items-center justify-center">
                  <ArrowRight class="text-muted-foreground/80" />
                </div>
                {{ navItem.title }} <ChevronRight /> {{ subItem.title }}
              </CommandItem>
            </template>
          </template>
        </CommandGroup>
        <CommandSeparator />
        <CommandGroup heading="Theme">
          <CommandItem
            value="light"
            @select="runCommand(() => (store = 'light'))"
          >
            <Sun /> <span>Light</span>
          </CommandItem>
          <CommandItem
            value="dard"
            @select="runCommand(() => (store = 'dark'))"
          >
            <Moon class="scale-90" />
            <span>Dark</span>
          </CommandItem>
          <CommandItem
            value="system"
            @select="runCommand(() => (store = 'auto'))"
          >
            <Laptop />
            <span>System</span>
          </CommandItem>
        </CommandGroup>
      </ScrollArea>
    </CommandList>
  </CommandDialog>
</template>
