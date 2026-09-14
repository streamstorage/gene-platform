import tailwindcss from '@tailwindcss/vite'
import vue from '@vitejs/plugin-vue'
import { fileURLToPath, URL } from 'node:url'
import { defineConfig } from 'vite'
import vueDevTools from 'vite-plugin-vue-devtools'

// https://vite.dev/config/
export default defineConfig(({ command }) => {
  const data = {
    plugins: [
      tailwindcss(),
      vue(),
      vueDevTools(), 
    ],
    resolve: {
      alias: {
        '@': fileURLToPath(new URL('./src', import.meta.url)),
      },
    },
  }
  
  if (command === 'serve') {
    return {
      ...data,
      server: {
        host: '0.0.0.0',
        proxy: {
          // Proxy all requests starting with /api
          '/api': {
            target: 'http://127.0.0.1:8080', // Replace with your backend API URL
            changeOrigin: true,
            secure: false,
          },
        },
      },
    }
  } else {
    return data
  }
})
