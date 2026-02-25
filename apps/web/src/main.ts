import { createApp } from 'vue'
import { createPinia } from 'pinia'
import './style.css'
import { initLocale } from './i18n'
import App from './App.vue'

async function bootstrap() {
  await initLocale()
  const app = createApp(App)
  app.use(createPinia())
  app.mount('#app')
}

bootstrap()
