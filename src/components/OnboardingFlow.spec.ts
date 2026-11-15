import { flushPromises, mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { afterEach, beforeEach, describe, expect, it } from 'vitest'
import { createMemoryHistory, createRouter } from 'vue-router'

import { i18n } from '@/i18n'
import { setDevTransport } from '@/lib/ipc'
import { useAppStore } from '@/stores/app'
import type {
  AppSettings,
  Bootstrap,
  CommandName,
  SettingsPatch,
  Workspace,
} from '@/types/ipc'
import OnboardingFlow from './OnboardingFlow.vue'

const timestamp = '2026-11-15T12:00:00.000Z'

function settings(): AppSettings {
  return {
    locale: 'en',
    theme: 'dark',
    sidebarCollapsed: false,
    lastRoute: '/my-issues',
    onboardingCompleted: false,
    detailPaneOpen: false,
    autoBackupEnabled: true,
    backupRetention: 20,
    window: { width: 1440, height: 900, x: null, y: null, maximized: false },
  }
}

describe('OnboardingFlow', () => {
  let workspace: Workspace | null
  let current: AppSettings
  let router: ReturnType<typeof createRouter>
  let wrapper: ReturnType<typeof mount> | null

  beforeEach(async () => {
    setActivePinia(createPinia())
    workspace = null
    current = settings()
    router = createRouter({
      history: createMemoryHistory(),
      routes: [
        { path: '/', redirect: '/my-issues' },
        {
          path: '/my-issues',
          component: { template: '<div />' },
        },
      ],
    })
    await router.push('/my-issues')
    await router.isReady()

    setDevTransport(async <T,>(command: CommandName, args?: Record<string, unknown>) => {
      const bootstrap = (): Bootstrap => ({
        appVersion: '0.1.0',
        schemaVersion: 1,
        dataDir: '/tmp/forgedesk',
        dbPath: '/tmp/forgedesk/forgedesk.db',
        backupsDir: '/tmp/forgedesk/backups',
        workspace,
        settings: current,
        backups: [],
        startupWarning: null,
      })
      switch (command) {
        case 'bootstrap':
          return bootstrap() as T
        case 'create_workspace':
          workspace = {
            id: 'ws-onboarding',
            name: String(args?.name),
            keyPrefix: 'FD',
            createdAt: timestamp,
            updatedAt: timestamp,
          }
          return structuredClone(workspace) as T
        case 'save_settings':
          current = { ...current, ...(args?.patch as SettingsPatch) }
          return structuredClone(current) as T
        case 'complete_onboarding':
          current = { ...current, onboardingCompleted: true }
          return structuredClone(current) as T
        default:
          throw { code: 'unsupported_capability', message: String(command) }
      }
    })

    const app = useAppStore()
    await app.load()
    i18n.global.locale.value = 'en'

    wrapper = mount(OnboardingFlow, {
      global: { plugins: [router, i18n] },
      attachTo: document.body,
    })
    await flushPromises()
  })

  afterEach(() => {
    wrapper?.unmount()
    wrapper = null
    document.body.innerHTML = ''
  })

  async function clickPrimaryFooterButton(): Promise<void> {
    const buttons = Array.from(
      document.querySelectorAll<HTMLButtonElement>('.fd-modal__footer button'),
    )
    buttons[buttons.length - 1]?.click()
    await flushPromises()
  }

  it('creates a workspace, saves preferences and completes onboarding', async () => {
    const app = useAppStore()
    expect(app.needsOnboarding).toBe(true)
    expect(document.body.textContent).toContain('Create your local workspace')

    const input = document.querySelector<HTMLInputElement>('.fd-input')
    expect(input).not.toBeNull()
    input!.value = 'Product Studio'
    input!.dispatchEvent(new Event('input'))

    await clickPrimaryFooterButton()
    expect(app.workspace?.name).toBe('Product Studio')
    expect(document.body.textContent).toContain('Make it yours')

    await clickPrimaryFooterButton()
    expect(current.locale).toBe('en')
    expect(current.theme).toBe('dark')
    expect(document.body.textContent).toContain('Connect GitHub')
    expect(document.body.textContent).toContain('Optional: finish setup now')
    expect(document.body.textContent).not.toContain('Stage 4')
    expect(document.body.textContent).not.toContain('not enabled in this build')
    const fakeConnect = Array.from(document.querySelectorAll('button')).filter(
      (button) => button.textContent?.trim() === 'Connect GitHub',
    )
    expect(fakeConnect).toHaveLength(0)

    i18n.global.locale.value = 'ru'
    await flushPromises()
    expect(document.body.textContent).toContain('Необязательно')
    expect(document.body.textContent?.toLowerCase()).not.toContain('этап 4')
    i18n.global.locale.value = 'en'
    await flushPromises()

    await clickPrimaryFooterButton()
    expect(current.onboardingCompleted).toBe(true)
    expect(app.needsOnboarding).toBe(false)
    expect(router.currentRoute.value.path).toBe('/my-issues')
  })
})
