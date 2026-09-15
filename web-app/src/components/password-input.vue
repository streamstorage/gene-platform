<script setup lang="ts">
  import { Button } from '@/components/ui/button'
  import { Input } from '@/components/ui/input'
  import { cn } from '@/lib/utils'
  import { Eye, EyeOff } from '@lucide/vue'
  import { useVModel } from '@vueuse/core'
  import { type HTMLAttributes, ref } from 'vue'

  // Prevents root div from auto-inheriting all fallthrough attributes
  defineOptions({
    inheritAttrs: false,
  })

  const props = defineProps<{
    disabled?: boolean
    modelValue?: string | number
    class?: HTMLAttributes['class']
  }>()

  const emits = defineEmits<{
    (e: 'update:modelValue', payload: string | number): void
  }>()

  const modelValue = useVModel(props, 'modelValue', emits, {
    passive: true,
  })

  const showPassword = ref(false)
</script>

<template>
  <div :class="cn('relative rounded-md', props.class)">
    <Input
      v-model="modelValue"
      :type="showPassword ? 'text' : 'password'"
      class="flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm shadow-xs transition-colors file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-muted-foreground focus-visible:ring-1 focus-visible:ring-ring focus-visible:outline-hidden disabled:cursor-not-allowed disabled:opacity-50"
      :disabled="disabled"
      v-bind="$attrs"
    />
    <Button
      type="button"
      size="icon"
      variant="ghost"
      :disabled="disabled"
      class="absolute inset-e-1 top-1/2 h-6 w-6 -translate-y-1/2 rounded-md text-muted-foreground"
      @click="showPassword = !showPassword"
    >
      <Eye
        v-if="showPassword"
        :size="18"
      />
      <EyeOff
        v-if="!showPassword"
        :size="18"
      />
      <span class="sr-only">
        {{ showPassword ? 'Hide password' : 'Show password' }}
      </span>
    </Button>
  </div>
</template>
