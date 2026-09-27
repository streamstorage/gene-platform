import { z } from 'zod'

const userRoleSchema = z.union([z.literal(2), z.literal(0)])
export type UserRole = z.infer<typeof userRoleSchema>

export const userSchema = z.object({
  id: z.number(),
  name: z.string(),
  email: z.string().email(),
  notes: z.string().nullable(),
  active: z.boolean(),
  role: userRoleSchema,
  last_seen: z
    .string()
    .nullable()
    .transform((str) => (str ? new Date(str + 'Z') : null)),
  created_at: z.string().transform((str) => new Date(str + 'Z')),
  updated_at: z.string().transform((str) => new Date(str + 'Z')),
})
export type User = z.infer<typeof userSchema>
