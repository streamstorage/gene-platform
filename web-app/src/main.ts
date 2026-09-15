import App from './App.vue'
import router from './router'
import './styles/index.css'
import { createPinia } from 'pinia'
import { configure } from 'vee-validate'
import { createApp } from 'vue'

configure({
  // validateOnBlur: false,        // Disables validation on blur
  validateOnChange: false, // Disables validation on change
  validateOnInput: false, // Disables validation on typing/input
  validateOnModelUpdate: false, // Disables validation on v-model updates
})

const app = createApp(App)

app.use(createPinia())
app.use(router)

app.mount('#app')
