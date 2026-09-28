import { authApi } from '@/api'
import { type User } from '@/views/admin/users/data/schema'
import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

const { postToken, deleteToken, getUserProfile } = authApi

export const useAuthStore = defineStore('auth', () => {
  const profile = ref<User | null>(null)
  const authenticated = computed(() => !!profile.value)
  const adminAuthorized = computed(() => !!profile.value && profile.value.role === 2)
  const user = computed(
    () => (profile.value ? { ...profile.value, avatar: '' } : {}) as User & { avatar: string }
  )

  async function signIn(email: string, password: string) {
    const value = await postToken(email, password)
    profile.value = value
    return value
  }
  async function signOut() {
    try {
      await deleteToken()
      profile.value = null
    } catch (error) {
      if (error instanceof Error) {
        if (error.name == '401') {
          profile.value = null
          return
        }
      }
      throw error
    }
  }
  async function getProfile() {
    try {
      profile.value = await getUserProfile()
    } catch (error) {
      if (error instanceof Error) {
        if (error.name == '401') {
          profile.value = null
          return
        }
      }
    }
  }
  return { adminAuthorized, authenticated, user, signIn, signOut, getProfile }
})
