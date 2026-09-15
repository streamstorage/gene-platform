<script setup lang="ts">
  import Logo from '@/assets/logo.vue'
</script>

<template>
  <div class="container grid h-svh max-w-none items-center justify-center">
    <div class="mx-auto flex w-full flex-col justify-center space-y-2 py-8 sm:p-8">
      <div class="mb-4 flex items-center justify-center">
        <Logo class="me-2" />
        <h1 class="text-xl font-medium">Gene Platform</h1>
      </div>
      <RouterView />
    </div>
  </div>
</template>
