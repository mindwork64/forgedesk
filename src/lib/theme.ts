import type { ResolvedTheme } from '@/types/ipc'

const THEME_HINT_KEY = 'fd.theme'

export function applyResolvedTheme(theme: ResolvedTheme): void {
  if (typeof document === 'undefined') {
    return
  }
  document.documentElement.dataset.theme = theme
  try {
    // Only a paint hint to avoid a flash before the bootstrap payload arrives.
    // SQLite remains the source of truth for the preference.
    localStorage.setItem(THEME_HINT_KEY, theme)
  } catch {
    // Storage can be unavailable in hardened webviews; the hint is optional.
  }
}

export function readThemeHint(): ResolvedTheme {
  try {
    const value = localStorage.getItem(THEME_HINT_KEY)
    return value === 'light' ? 'light' : 'dark'
  } catch {
    return 'dark'
  }
}
