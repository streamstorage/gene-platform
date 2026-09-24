<script setup lang="ts">
  import ConfirmDialog from './ConfirmDialog.vue'
  import { useRouter } from 'vue-router'

  const modelValue = defineModel<boolean>()

  const router = useRouter()
  const handleSignOut = () => {
    modelValue.value = false
    router.push('/')
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
