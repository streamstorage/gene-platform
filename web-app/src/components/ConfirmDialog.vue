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
  import type { Component, HTMLAttributes } from 'vue'

  const open = defineModel<boolean>()

  type ConfirmDialogProps = {
    title: string | Component
    disabled?: boolean
    desc: string | Component
    cancelBtnText?: string
    confirmText?: string | Component
    destructive?: boolean
    isLoading?: boolean
    class?: HTMLAttributes['class']
    children?: Component
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
    <AlertDialogContent :class="cn(props.class && props.class)">
      <AlertDialogHeader class="text-start">
        <AlertDialogTitle>{{ title }}</AlertDialogTitle>
        <AlertDialogDescription as-child>
          <div>{{ desc }}</div>
        </AlertDialogDescription>
      </AlertDialogHeader>
      {{ children }}
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
          {{ confirmText }}
        </Button>
      </AlertDialogFooter>
    </AlertDialogContent>
  </AlertDialog>
</template>
