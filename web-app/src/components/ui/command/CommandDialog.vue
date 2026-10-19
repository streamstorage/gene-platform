<script setup lang="ts">
  import type { DialogRootEmits, DialogRootProps } from 'reka-ui'
  import { useForwardPropsEmits } from 'reka-ui'
  import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogHeader,
    DialogTitle,
  } from '@/components/ui/dialog'
  import Command from './Command.vue'

  const props = withDefaults(
    defineProps<
      DialogRootProps & {
        title?: string
        description?: string
      }
    >(),
    {
      title: 'Command Palette',
      description: 'Search for a command to run...',
    }
  )
  const emits = defineEmits<DialogRootEmits>()

  const forwarded = useForwardPropsEmits(props, emits)
</script>

<template>
  <Dialog
    v-slot="slotProps"
    v-bind="forwarded"
  >
    <DialogContent class="overflow-hidden p-0">
      <DialogHeader class="sr-only">
        <DialogTitle>{{ title }}</DialogTitle>
        <DialogDescription>{{ description }}</DialogDescription>
      </DialogHeader>
      <Command class="**:data-[slot=command-input-wrapper]:h-12">
        <slot v-bind="slotProps" />
      </Command>
    </DialogContent>
  </Dialog>
</template>
