import api, { error } from './api'
import { type User, userSchema } from '@/views/admin/users/data/schema'

export const authApi = {
  postToken: async (email: string, password: string) => {
    const headers = new Headers()
    headers.append('Content-Type', 'application/json')

    const response = await fetch(api.token(), {
      method: 'POST',
      body: JSON.stringify({
        email,
        password,
      }),
      headers,
    })
    if (!response.ok) {
      throw await error(response)
    }
    const profile = userSchema.parse(await response.json())
    return profile
  },

  deleteToken: async () => {
    const response = await fetch(api.token(), { method: 'DELETE' })
    if (!response.ok) {
      throw await error(response)
    }
    return
  },

  getProfile: async () => {
    const response = await fetch(api.profile())
    if (!response.ok) {
      throw await error(response)
    }
    const profile = userSchema.parse(await response.json())
    return profile
  },
  updateProfile: async (user: User & { password: string; newPassword: string }) => {
    const headers = new Headers()
    headers.append('Content-Type', 'application/json')

    const response = await fetch(api.profile(), {
      method: 'PUT',
      body: JSON.stringify(user),
      headers,
    })
    if (!response.ok) {
      throw await error(response)
    }
    return
  },
}
