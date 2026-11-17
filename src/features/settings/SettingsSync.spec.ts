import { flushPromises, mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { afterEach, describe, expect, it } from 'vitest'
import { createMemoryHistory, createRouter } from 'vue-router'

import { i18n } from '@/i18n'
import { setDevTransport } from '@/lib/ipc'
import type { ApiMonitorState, GitHubConnection } from '@/types/ipc'
import SettingsSync from './SettingsSync.vue'

const timestamp = '2026-11-16T12:00:00.000Z'

const emptyMonitor: ApiMonitorState = {
  connected: false,
  accountLogin: null,
  budgets: [],
  requests: [],
  queue: [],
  control: { paused: false, reason: '', updatedAt: timestamp },
  lastObservation: null,
}

function connection(): GitHubConnection {
  return {
    connected: true,
    login: 'mindwork64',
    userId: 34349377,
    scopes: '',
    expiresAt: timestamp,
    refreshExpiresAt: timestamp,
    storage: 'keyring',
    storageWarning: null,
  }
}

describe('SettingsSync', () => {
  let wrapper: ReturnType<typeof mount> | null = null
  let router: ReturnType<typeof createRouter>

  async function mountSync(
    locale: 'en' | 'ru' = 'en',
    connected = false,
  ): Promise<void> {
    setActivePinia(createPinia())
    i18n.global.locale.value = locale
    setDevTransport(async <T,>(command: string) => {
      if (command === 'get_api_monitor') {
        return structuredClone(emptyMonitor) as T
      }
      if (command === 'github_connection') {
        return (connected ? connection() : null) as T
      }
      if (command === 'github_sync_conflicts') {
        return [] as T
      }
      throw { code: 'unsupported_capability', message: String(command) }
    })

    router = createRouter({
      history: createMemoryHistory(),
      routes: [
        { path: '/settings/sync', component: { template: '<div />' } },
        { path: '/integrations', component: { template: '<div />' } },
        { path: '/api-monitor', component: { template: '<div />' } },
      ],
    })
    await router.push('/settings/sync')
    await router.isReady()
    wrapper = mount(SettingsSync, {
      global: { plugins: [router, i18n] },
      attachTo: document.body,
    })
    await flushPromises()
  }

  afterEach(() => {
    wrapper?.unmount()
    wrapper = null
    document.body.innerHTML = ''
    i18n.global.locale.value = 'en'
  })

  it('describes background sync as unavailable instead of claiming GitHub is absent', async () => {
    await mountSync()
    const text = wrapper?.text() ?? ''
    expect(text).toContain('Background synchronization')
    expect(text).toContain('Not implemented')
    expect(text).toContain('Manual actions only')
    expect(text).not.toContain('arrives with GitHub integration')
    expect(text).toContain('Observed buckets: 0')
  })

  it('shows the connected GitHub account', async () => {
    await mountSync('en', true)
    expect(wrapper?.text()).toContain('Connected as mindwork64')
  })

  it('localizes the status in Russian', async () => {
    await mountSync('ru')
    expect(wrapper?.text()).toContain('Фоновая синхронизация')
    expect(wrapper?.text()).toContain('Не реализована')
    expect(wrapper?.text()).toContain('Действия только вручную')
  })

  it('opens the API monitor and integrations screens', async () => {
    await mountSync()
    const apiButton = wrapper?.findAll('button').find((button) => button.text().includes('Open API monitor'))
    await apiButton?.trigger('click')
    await flushPromises()
    expect(router.currentRoute.value.path).toBe('/api-monitor')

    await router.push('/settings/sync')
    const integrationsButton = wrapper?.findAll('button').find((button) => button.text().includes('Open integrations'))
    await integrationsButton?.trigger('click')
    await flushPromises()
    expect(router.currentRoute.value.path).toBe('/integrations')
  })
})
