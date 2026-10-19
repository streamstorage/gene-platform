import { useNProgress } from '@vueuse/integrations/useNProgress'
import type { Router } from 'vue-router'

const { start, done } = useNProgress(0.0, {
  speed: 500,
  trickleSpeed: 200,
  showSpinner: false,
})

/**
 * global router guard
 * now only used for progress bar
 */
export function setupCommonGuard(router: Router) {
  router.beforeEach(() => {
    start()
  })

  router.afterEach(() => {
    done()
  })
}
