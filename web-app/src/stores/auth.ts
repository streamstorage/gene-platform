import { type Profile } from '@/api/auth-api'
import { ref, computed } from 'vue'
import { defineStore } from 'pinia'

export const useAuthStore = defineStore('auth', () => {
  const authenticated = ref(true)
  const user = ref<Profile>({
    avatar: '',
    name: 'Luis Liu',
    email: 'vangork@live.com',
    role: 2,
  } as Profile)

  const count = ref(0)
  const doubleCount = computed(() => count.value * 2)
  function increment() {
    count.value++
  }

  return { authenticated, user, count, doubleCount, increment }
})
