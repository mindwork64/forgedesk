import { flushPromises, mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { afterEach, describe, expect, it } from 'vitest'

import { i18n } from '@/i18n'
import { setDevTransport } from '@/lib/ipc'
import { useAppStore } from '@/stores/app'
import type { Bootstrap } from '@/types/ipc'
import SettingsWorkspace from './SettingsWorkspace.vue'

const timestamp = '2026-11-17T00:00:00.000Z'

function bootstrap(): Bootstrap {
  return {
    appVersion: '0.1.0',
    schemaVersion: 11,
    dataDir: '/data',
    dbPath: '/data/forgedesk.db',
    backupsDir: '/data/backups',
    workspace: {
      id: 'workspace-1',
      name: 'Workspace',
      keyPrefix: 'FD',
      createdAt: timestamp,
      updatedAt: timestamp,
    },
    settings: {
      locale: 'en',
      theme: 'dark',
      sidebarCollapsed: false,
      lastRoute: '/settings',
      onboardingCompleted: true,
      detailPaneOpen: true,
      autoBackupEnabled: false,
      backupRetention: 20,
      window: {
        width: 1440,
        height: 900,
        x: null,
        y: null,
        maximized: false,
      },
    },
    backups: [],
    startupWarning: null,
  }
}

describe('SettingsWorkspace', () => {
  let wrapper: ReturnType<typeof mount> | null = null

  afterEach(() => {
    wrapper?.unmount()
    wrapper = null
    document.body.innerHTML = ''
    i18n.global.locale.value = 'en'
  })

  it('saves and displays the normalized workspace issue prefix', async () => {
    const pinia = createPinia()
    setActivePinia(pinia)
    i18n.global.locale.value = 'en'
    const app = useAppStore()
    app.$patch({ data: bootstrap() })

    const updates: string[] = []
    setDevTransport(async <T,>(command: string, args?: Record<string, unknown>) => {
      if (command !== 'update_workspace_key_prefix') {
        throw { code: 'unsupported_capability', message: command }
      }
      const prefix = String(args?.prefix ?? '')
      updates.push(prefix)
      return { ...app.workspace!, keyPrefix: prefix.trim().toUpperCase() } as T
    })

    wrapper = mount(SettingsWorkspace, {
      global: { plugins: [pinia, i18n] },
      attachTo: document.body,
    })
    await flushPromises()

    await wrapper.get('input').setValue('exhi')
    await wrapper.get('form').trigger('submit')
    await flushPromises()

    expect(updates).toEqual(['exhi'])
    expect(app.workspace?.keyPrefix).toBe('EXHI')
    expect((wrapper.get('input').element as HTMLInputElement).value).toBe('EXHI')
  })
})
