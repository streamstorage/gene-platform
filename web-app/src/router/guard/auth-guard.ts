import { useAuthStore } from '@/stores'
import { storeToRefs } from 'pinia'
import type { Router } from 'vue-router'

export function setupAuthGuard(router: Router) {
  router.beforeEach(async (to, _from) => {
    const authStore = useAuthStore()
    const { adminAuthorized, authenticated } = storeToRefs(authStore)

    await authStore.getProfile()

    if (
      (to.meta && to.meta.guard == 'guest' && authenticated.value) ||
      (to.meta && to.meta.guard == 'user' && !authenticated.value) ||
      (to.meta && to.meta.guard == 'admin' && (!authenticated.value || !adminAuthorized.value))
    ) {
      return { name: 'Index' }
    }
  })
}
