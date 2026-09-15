<script setup lang="ts">
  import { Button } from '@/components/ui/button'
  import { Dna } from '@lucide/vue'
</script>

<template>
  <div class="h-svh">
    <div class="m-auto flex h-full w-full flex-col items-center justify-center gap-2">
      <Dna :size="72" />
      <h1 class="text-4xl leading-tight font-bold">Gene Platform</h1>
      <p class="text-center text-muted-foreground">Click the button below to start</p>
      <Button
        as-child
        class="mt-4"
      >
        <RouterLink :to="{ name: 'sign-in' }">Sign In</RouterLink>
      </Button>
    </div>
  </div>
</template>
