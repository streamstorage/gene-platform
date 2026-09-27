import type { UserRole } from './schema'
import type { FacetedFilterOption } from '@/components/data-table'
import { Shield, UserCheck } from '@lucide/vue'

export const status = [
  {
    label: 'Active',
    value: true,
    style: 'bg-teal-100/30 text-teal-900 dark:text-teal-200 border-teal-200',
  },
  { label: 'Inactive', value: false, style: 'bg-neutral-300/40 border-neutral-300' },
] satisfies (FacetedFilterOption<boolean> & { style: string })[]

export const roles = [
  {
    label: 'Admin',
    value: 2,
    icon: Shield,
  },
  {
    label: 'User',
    value: 0,
    icon: UserCheck,
  },
] satisfies FacetedFilterOption<UserRole>[]
