<script setup lang="ts">
  import { Dna } from '@lucide/vue'
</script>

<template>
  <div class="container grid h-svh max-w-none items-center justify-center">
    <div class="mx-auto flex w-full flex-col justify-center space-y-2 py-8 sm:p-8">
      <div class="mb-4 flex items-center justify-center">
        <Dna
          class="me-2"
          :size="24"
        />
        <h1 class="text-xl font-medium">Gene Platform</h1>
      </div>
      <RouterView />
    </div>
  </div>
</template>
