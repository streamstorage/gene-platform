import type { UserRole, UserStatus } from './schema'
import type { FacetedFilterOption } from '@/components/data-table'
import { Shield, UserCheck } from '@lucide/vue'

export const statusColor = new Map<UserStatus, string>([
  ['active', 'bg-teal-100/30 text-teal-900 dark:text-teal-200 border-teal-200'],
  ['inactive', 'bg-neutral-300/40 border-neutral-300'],
])

export const status = [
  { label: 'Active', value: 'active' },
  { label: 'Inactive', value: 'inactive' },
] satisfies FacetedFilterOption<UserStatus>[]

export const roles = [
  {
    label: 'Admin',
    value: 'admin',
    icon: Shield,
  },
  {
    label: 'User',
    value: 'user',
    icon: UserCheck,
  },
] satisfies FacetedFilterOption<UserRole>[]
