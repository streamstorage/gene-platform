import App from './App.vue'
import { setupPlugins } from './plugins'
import './styles/index.css'
import { createApp } from 'vue'

const app = createApp(App)

setupPlugins(app)

app.mount('#app')
