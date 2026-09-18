<script setup lang="ts">
  import { Button } from '@/components/ui/button'
  import { Dna, LogIn } from '@lucide/vue'
  import { useAuthStore } from '@/stores/auth'
  import { storeToRefs } from 'pinia'
  import { useRouter } from 'vue-router'

  const router = useRouter()
  const authStore = useAuthStore()

  const { authenticated } = storeToRefs(authStore)
</script>

<template>
  <div class="h-svh">
    <div class="m-auto flex h-full w-full flex-col items-center justify-center gap-2">
      <Dna :size="72" />
      <h1 class="text-4xl leading-tight font-bold">Gene Platform</h1>
      <p class="text-center text-muted-foreground">Click the button below to start</p>

      <Button
        class="mt-4"
        @click="router.push({ name: authenticated ? 'Dashboard' : 'SignIn' })"
      >
        <LogIn />
        {{ authenticated ? 'Enter' : 'Sign in' }}
      </Button>
    </div>
  </div>
</template>
