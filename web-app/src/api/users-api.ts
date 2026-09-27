import api, { arrayFromFallible, error } from './api'
import { userSchema } from '@/views/admin/users/data/schema'

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
}
