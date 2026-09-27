import App from './App.vue'
import { setupPlugins } from './plugins'
import router from './router'
import stores from './stores'
import './styles/index.css'
import { createApp } from 'vue'

const app = createApp(App)

app.use(stores)
app.use(router)

setupPlugins()

app.mount('#app')
