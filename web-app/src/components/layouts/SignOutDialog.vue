<script setup lang="ts">
  import ConfirmDialog from '@/components/ConfirmDialog.vue'
  import { useAuthStore } from '@/stores'
  import { useRouter } from 'vue-router'
  import { toast } from 'vue-sonner'

  const modelValue = defineModel<boolean>()

  const { push } = useRouter()
  const authStore = useAuthStore()

  const handleSignOut = async () => {
    modelValue.value = false
    try {
      await authStore.signOut()
      push('/')
    } catch (err) {
      toast.error(err instanceof Error ? `${err.name}: ${err.message}` : String(err))
    }
  }
</script>

<template>
  <ConfirmDialog
    v-model="modelValue"
    title="Sign out"
    desc="Are you sure to sign out? You will need to sign in again to access the application."
    confirmText="Sign out"
    destructive
    :handleConfirm="handleSignOut"
    class="sm:max-w-sm"
  />
</template>
