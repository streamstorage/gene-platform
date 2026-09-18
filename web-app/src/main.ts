import App from './App.vue'
import { setupPlugins } from './plugins'
import router from './router'
import './styles/index.css'
import { createPinia } from 'pinia'
import { createApp } from 'vue'

const app = createApp(App)
app.use(createPinia())
app.use(router)
setupPlugins()

app.mount('#app')
