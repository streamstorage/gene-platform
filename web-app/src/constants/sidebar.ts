import { type NavGroup } from '@/components/layouts/app-sidebar'
import { Construction, HelpCircle, LayoutDashboard, Palette, Settings, UserCog } from '@lucide/vue'

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
    title: 'Other',
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
            title: 'Appearance',
            url: '/settings/appearance',
            icon: Palette,
          },
        ],
      },
      {
        title: 'Help Center',
        url: '/help-center',
        icon: HelpCircle,
      },
    ],
  },
]
