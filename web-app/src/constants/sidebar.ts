import { type NavGroup } from '@/components/layouts/app-sidebar'
import {
  CircleChevronLeft,
  Columns3Cog,
  Construction,
  Dices,
  Flower,
  LayoutDashboard,
  Palette,
  Settings,
  UserCog,
  Users,
} from '@lucide/vue'

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
      {
        title: 'Secured by Clerk',
        icon: Construction,
        items: [
          {
            title: 'Sign In',
            url: '/clerk/sign-in',
          },
          {
            title: 'Sign Up',
            url: '/clerk/sign-up',
          },
        ],
      },
    ],
  },
  {
    title: 'User',
    items: [
      {
        title: 'Settings',
        icon: Settings,
        items: [
          {
            title: 'Profile',
            url: '/settings',
            icon: UserCog,
          },
          {
            title: 'Preference',
            url: '/settings/appearance',
            icon: Palette,
          },
        ],
      },
    ],
  },
  {
    title: 'Other',
    items: [
      {
        title: 'System Administration',
        url: '/admin/summary',
        icon: Columns3Cog,
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
  {
    title: 'Other',
    items: [
      {
        title: 'Back to application',
        url: '/app/dashboard',
        icon: CircleChevronLeft,
      },
    ],
  },
]
