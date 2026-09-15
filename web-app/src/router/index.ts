import AuthLayout from '@/layouts/auth-layout.vue'
import { useNProgress } from '@vueuse/integrations/useNProgress'
import { createRouter, createWebHistory, type RouteRecordRaw } from 'vue-router'

const { start, done } = useNProgress(0.0, {
  speed: 500,
  trickleSpeed: 200,
  showSpinner: false,
})

const routes: Array<RouteRecordRaw> = [
  {
    path: '/:pathMatch(.*)*',
    redirect: { name: '404' },
  },
  {
    path: '/auth',
    component: AuthLayout,
    meta: {
      guard: 'guest',
    },
    children: [
      {
        name: 'sign-in',
        path: 'sign-in',
        component: () => import('@/pages/auth/sign-in.vue'),
      },
    ],
  },
  {
    name: '404',
    path: '/404',
    component: () => import('@/pages/errors/404.vue'),
  },
  {
    name: '503',
    path: '/503',
    component: () => import('@/pages/errors/503.vue'),
  },
  {
    name: 'index',
    path: '/',
    component: () => import('@/pages/HelloWorld.vue'),
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

router.beforeEach(() => {
  start()
})

router.afterEach(() => {
  done()
})

export default router
