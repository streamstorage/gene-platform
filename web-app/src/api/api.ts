import { z } from 'zod'

const DROPPED = Symbol('dropped')

export function arrayFromFallible<T extends z.ZodType>(schema: T) {
  return z
    .array(schema.catch(DROPPED as never))
    .transform((items) => items.filter((item) => item !== DROPPED) as z.output<T>[])
}

export const error = async (resp: Response) => {
  const error = new Error(`${await resp.text()}`)
  error.name = `${resp.status}`
  return error
}

const apiBaseUrl = import.meta.env.VITE_API_BASE_URL

export default {
  token: () => `${apiBaseUrl}/api/auth/token`,
  profile: () => `${apiBaseUrl}/api/user/profile`,

  users: () => `${apiBaseUrl}/api/admin/users`,
  user: (id: number) => `${apiBaseUrl}/api/admin/users/${id}`,
}
