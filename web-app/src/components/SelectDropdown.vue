<script setup lang="ts">
  import { cn } from '@/lib/utils'
  import { FormControl } from '@/components/ui/form'
  import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
  } from '@/components/ui/select'
  import { Loader } from '@lucide/vue'

  import type { SelectRootEmits, SelectRootProps } from 'reka-ui'
  import { useForwardPropsEmits } from 'reka-ui'
  import { type HTMLAttributes } from 'vue'

  type SelectDropdownProps = SelectRootProps & {
    placeholder?: string
    isPending?: boolean
    items: { label: string; value: string | number }[] | undefined
    disabled?: boolean
    class?: HTMLAttributes['class']
  }
  const props = withDefaults(defineProps<SelectDropdownProps>(), {
    class: '',
  })
  const emits = defineEmits<SelectRootEmits>()
  const forwarded = useForwardPropsEmits(props, emits)
</script>

<template>
  <Select v-bind="forwarded">
    <FormControl>
      <SelectTrigger
        :disabled="disabled"
        :class="cn(props.class)"
      >
        <SelectValue :placeholder="placeholder ?? 'Select'" />
      </SelectTrigger>
    </FormControl>
    <SelectContent>
      <SelectItem
        v-if="isPending"
        disabled
        value="loading"
        class="h-14"
      >
        <div class="flex items-center justify-center gap-2">
          <Loader class="h-5 w-5 animate-spin" />
          {{ '  ' }}
          Loading...
        </div>
      </SelectItem>
      <template v-else>
        <SelectItem
          v-for="item in items"
          :key="item.value"
          :value="item.value"
        >
          {{ item.label }}
        </SelectItem>
      </template>
    </SelectContent>
  </Select>
</template>
