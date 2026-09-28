import api, { arrayFromFallible, error } from './api'
import { type User, userSchema } from '@/views/admin/users/data/schema'

export const usersApi = {
  listAll: async () => {
    const response = await fetch(api.users())
    if (!response.ok) {
      throw await error(response)
    }
    const Users = arrayFromFallible(userSchema)
    const items = Users.parse(await response.json())

    return items
  },

  addUser: async (user: User) => {
    const headers = new Headers()
    headers.append('Content-Type', 'application/json')

    const response = await fetch(api.users(), { method: 'POST', body: JSON.stringify(user), headers })
    if (!response.ok) {
      throw await error(response)
    }
    return
  },

  updateUser: async (user: User) => {
    const headers = new Headers()
    headers.append('Content-Type', 'application/json')

    const response = await fetch(api.user(user.id), { method: 'PUT', body: JSON.stringify(user), headers })
    if (!response.ok) {
      throw await error(response)
    }
    return
  }
}
