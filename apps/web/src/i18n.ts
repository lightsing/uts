import { i18n } from '@lingui/core'
import { messages as enMessages } from './locales/en.po'

i18n.load('en', enMessages)
i18n.activate('en')

export async function initLocale() {
  const saved = localStorage.getItem('uts-locale')
  if (saved && saved !== 'en') {
    try {
      const catalog = await import(`./locales/${saved}.po`)
      i18n.load(saved, catalog.messages)
      i18n.activate(saved)
    } catch {
      // fallback to English
    }
  }
}

export { i18n }
