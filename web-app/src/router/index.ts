import AppLayout from '@/components/layouts/AppLayout.vue'
import AuthLayout from '@/components/layouts/AuthLayout.vue'
import { useNProgress } from '@vueuse/integrations/useNProgress'
import { createRouter, createWebHistory, type RouteRecordRaw } from 'vue-router'

const routes: Array<RouteRecordRaw> = [
  {
    path: '/',
    name: 'Index',
    component: () => import('@/views/Index.vue'),
  },
  {
    path: '/auth',
    component: AuthLayout,
    meta: {
      guard: 'guest',
    },
    children: [
      {
        path: '/auth/sign-in',
        name: 'SignIn',
        component: () => import('@/views/auth/SignIn.vue'),
      },
    ],
  },
  {
    path: '/app',
    name: 'app',
    component: AppLayout,
    props: { isAdmin: false },
    meta: {
      guard: 'user',
    },
    redirect: { name: 'Dashboard' },
    children: [
      {
        path: '/app/dashboard',
        name: 'Dashboard',
        component: () => import('@/views/app/Dashboard.vue'),
      },
    ],
  },
  {
    path: '/503',
    name: '503',
    component: () => import('@/views/errors/503.vue'),
  },
  {
    path: '/:pathMatch(.*)*',
    name: '404',
    component: () => import('@/views/errors/404.vue'),
  },
]

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes,
  scrollBehavior(to, _from, savedPosition) {
    if (savedPosition) {
      return savedPosition
    }
    // For some reason using documentation example doesn't scroll on page navigation.
    if (to.hash) {
      return { el: to.hash, behavior: 'smooth' }
    } else {
      window.scrollTo(0, 0)
    }
  },
})

const { start, done } = useNProgress(0.0, {
  speed: 500,
  trickleSpeed: 200,
  showSpinner: false,
})

router.beforeEach(() => {
  start()
})

router.afterEach(() => {
  done()
})

export default router
