import { setupPinia } from './pinia/setup'
import { setupRouter } from './router/setup'
import { setupVeeValidate } from './vee-validate/setup'
import type { App } from 'vue'

export function setupPlugins(app: App) {
  setupPinia(app)
  setupRouter(app)
  setupVeeValidate()
}
