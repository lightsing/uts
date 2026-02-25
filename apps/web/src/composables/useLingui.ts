import { ref } from 'vue'
import { i18n } from '@/i18n'

const _locale = ref(i18n.locale || 'en')

// Keep _locale in sync when initLocale restores a saved locale
if (i18n.locale && i18n.locale !== _locale.value) {
  _locale.value = i18n.locale
}

export function useLingui() {
  /**
   * Translate a message. Accesses the reactive `_locale` ref
   * so that Vue re-renders when the active locale changes.
   */
  function t(id: string, values?: Record<string, unknown>): string {
    // Reading _locale.value ensures Vue tracks this as a dependency
    // so template expressions re-evaluate when locale changes.
    void _locale.value
    return i18n._(id, values)
  }

  async function activate(locale: string) {
    const catalog = await import(`../locales/${locale}.po`)
    i18n.load(locale, catalog.messages)
    i18n.activate(locale)
    _locale.value = locale
    localStorage.setItem('uts-locale', locale)
  }

  return { t, locale: _locale, activate, i18n }
}
