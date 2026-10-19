import { type NavGroup } from '@/components/layouts/app-sidebar'
import { Dices, Flower, LayoutDashboard, Palette, Settings, UserCog, Users } from '@lucide/vue'

export const navGroups: NavGroup[] = [
  {
    title: 'General',
    items: [
      {
        title: 'Dashboard',
        url: '/app/dashboard',
        icon: LayoutDashboard,
      },
      {
        title: 'Samples',
        url: '/app/samples',
        icon: Dices,
      },
    ],
  },
  {
    title: 'Other',
    items: [
      {
        title: 'Settings',
        icon: Settings,
        items: [
          {
            title: 'Profile',
            url: '/app/settings/profile',
            icon: UserCog,
          },
          {
            title: 'Preference',
            url: '/app/settings/preference',
            icon: Palette,
          },
        ],
      },
    ],
  },
]

export const navAdminGroups: NavGroup[] = [
  {
    title: 'General',
    items: [
      {
        title: 'Summary',
        url: '/admin/summary',
        icon: Flower,
      },
      {
        title: 'Users',
        url: '/admin/users',
        icon: Users,
      },
    ],
  },
]
