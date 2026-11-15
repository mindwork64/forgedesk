import { createI18n } from 'vue-i18n'

import type { LocalePreference, ResolvedLocale, ThemePreference } from '@/types/ipc'
import en from './en'
import ru from './ru'

export const SUPPORTED_LOCALES: ResolvedLocale[] = ['en', 'ru']

export function resolveLocale(preference: LocalePreference): ResolvedLocale {
  if (preference === 'en' || preference === 'ru') {
    return preference
  }
  const language =
    typeof navigator !== 'undefined' ? navigator.language.toLowerCase() : 'en'
  return language.startsWith('ru') ? 'ru' : 'en'
}

export function resolveTheme(
  preference: ThemePreference,
  systemPrefersDark: boolean,
): 'dark' | 'light' {
  if (preference === 'dark' || preference === 'light') {
    return preference
  }
  return systemPrefersDark ? 'dark' : 'light'
}

export const i18n = createI18n({
  legacy: false,
  locale: 'en',
  fallbackLocale: 'en',
  messages: { en, ru },
  missingWarn: false,
  fallbackWarn: false,
})

export function applyLocale(locale: ResolvedLocale): void {
  i18n.global.locale.value = locale
  if (typeof document !== 'undefined') {
    document.documentElement.lang = locale
  }
}
