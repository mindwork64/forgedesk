import { createPinia } from 'pinia'
import { createApp } from 'vue'

import App from './App.vue'
import { tooltip } from './directives/tooltip'
import { i18n } from './i18n'
import { applyResolvedTheme, readThemeHint } from './lib/theme'
import { router } from './router'
import './styles/base.css'

// Paint hint only; SQLite settings remain authoritative after bootstrap.
applyResolvedTheme(readThemeHint())

async function start(): Promise<void> {
  if (import.meta.env.VITE_VISUAL_HARNESS === 'true') {
    const { installVisualHarness } = await import('./dev/visualHarness')
    installVisualHarness()
  }

  const app = createApp(App)
  app.use(createPinia())
  app.use(router)
  app.use(i18n)
  app.directive('tooltip', tooltip)

  app.config.errorHandler = (error, _instance, info) => {
    // No crash reporter, no telemetry. Surface the failure in local console for
    // diagnostics; the UI remains usable where possible.
    console.error('[ForgeDesk] Unhandled UI error', info, error)
  }

  await router.isReady()
  app.mount('#app')
}

void start()
