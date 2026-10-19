<script setup lang="ts">
  import { cn } from '@/lib/utils'
  import {
    AlertDialog,
    AlertDialogCancel,
    AlertDialogContent,
    AlertDialogDescription,
    AlertDialogFooter,
    AlertDialogHeader,
    AlertDialogTitle,
  } from '@/components/ui/alert-dialog'
  import { Button } from '@/components/ui/button'
  import type { HTMLAttributes } from 'vue'

  const open = defineModel<boolean>()

  type ConfirmDialogProps = {
    title?: string
    disabled?: boolean
    desc?: string
    cancelBtnText?: string
    confirmText?: string
    destructive?: boolean
    isLoading?: boolean
    class?: HTMLAttributes['class']
  } & (
    { form: string; handleConfirm?: undefined } | { form?: undefined; handleConfirm: () => void }
  )

  const props = withDefaults(defineProps<ConfirmDialogProps>(), {
    cancelBtnText: 'Cancel',
    confirmText: 'Continue',
  })
</script>

<template>
  <AlertDialog v-model:open="open">
    <AlertDialogContent :class="cn(props.class)">
      <AlertDialogHeader class="text-start">
        <AlertDialogTitle>
          <slot name="title">
            {{ title }}
          </slot>
        </AlertDialogTitle>
        <AlertDialogDescription as-child>
          <slot name="description">
            <div>{{ desc }}</div>
          </slot>
        </AlertDialogDescription>
      </AlertDialogHeader>
      <slot></slot>
      <AlertDialogFooter>
        <AlertDialogCancel :disabled="isLoading">
          {{ cancelBtnText }}
        </AlertDialogCancel>
        <Button
          :type="form ? 'submit' : 'button'"
          :form="form"
          @click="handleConfirm"
          :variant="destructive ? 'destructive' : 'default'"
          :disabled="disabled || isLoading"
        >
          <slot name="confirm">
            {{ confirmText }}
          </slot>
        </Button>
      </AlertDialogFooter>
    </AlertDialogContent>
  </AlertDialog>
</template>
