import { createI18n } from 'vue-i18n'
import en_US from './en-US'
import zh_CN from './zh-CN'

const i18n = createI18n({
  locale: localStorage.getItem('locale') || 'en-US',
  fallbackLocale: 'en-US',
  legacy: false,
  messages: {
    'en-US': en_US,
    'zh-CN': zh_CN,
  },
})

export default i18n
