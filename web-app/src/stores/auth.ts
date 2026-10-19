import { authApi } from '@/api'
import { type User } from '@/views/admin/users/data/schema'
import { getDisplayNameInitials } from '@/lib/utils'
import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

const { postToken, deleteToken, getProfile, updateProfile } = authApi

export const useAuthStore = defineStore('auth', () => {
  const profile = ref<User | null>(null)
  const authenticated = computed(() => !!profile.value)
  const adminAuthorized = computed(() => !!profile.value && profile.value.role === 2)
  const user = computed(
    () =>
      (profile.value
        ? { ...profile.value, avatar: '', initials: getDisplayNameInitials(profile.value.name) }
        : {
            avatar: '',
          }) as User & { avatar: string; initials: string }
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
  async function getUserProfile() {
    try {
      profile.value = await getProfile()
    } catch (error) {
      if (error instanceof Error) {
        if (error.name == '401') {
          profile.value = null
        }
      } else {
        console.log(error)
      }
    }
  }
  async function updateUserProfile(user: User & { password: string; newPassword: string }) {
    await updateProfile(user)
    await getUserProfile()
  }

  return {
    adminAuthorized,
    authenticated,
    user,
    signIn,
    signOut,
    getUserProfile,
    updateUserProfile,
  }
})
