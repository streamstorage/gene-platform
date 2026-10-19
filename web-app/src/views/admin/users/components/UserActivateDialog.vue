<script setup lang="ts">
  import { usersApi } from '@/api'
  import type { User } from '../data/schema'
  import ConfirmDialog from '@/components/ConfirmDialog.vue'
  import { computed, ref } from 'vue'
  import { toast } from 'vue-sonner'

  const modelValue = defineModel<boolean>()

  const props = defineProps<{
    activate: boolean
    users: Array<User>
  }>()

  const emit = defineEmits(['updated'])

  const action = computed(() => (props.activate ? 'Acticate' : 'Deactivate'))
  const userCount = computed(() => props.users.length)

  const isLoading = ref(false)
  const { activateUsers, deactivateUsers } = usersApi

  const handleConfirm = async () => {
    isLoading.value = true
    try {
      if (props.activate) {
        await activateUsers(props.users)
      } else {
        await deactivateUsers(props.users)
      }
      emit('updated')
      modelValue.value = false
    } catch (err) {
      toast.error(err instanceof Error ? `${err.name}: ${err.message}` : String(err))
    } finally {
      isLoading.value = false
    }
  }
</script>

<template>
  <ConfirmDialog
    v-model="modelValue"
    :title="action"
    :desc="`Are you sure to ${action} ${userCount > 1 ? `the ${userCount} users` : `user: ${props.users[0].name}`} ?`"
    :confirmText="action"
    :handleConfirm="handleConfirm"
    class="sm:max-w-sm"
    :is-loading="isLoading"
  />
</template>
