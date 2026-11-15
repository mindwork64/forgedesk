import { flushPromises, mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { afterEach, describe, expect, it, vi } from 'vitest'
import { createMemoryHistory, createRouter } from 'vue-router'

import { i18n } from '@/i18n'
import { setDevTransport } from '@/lib/ipc'
import IntegrationsView from './IntegrationsView.vue'

vi.mock('@/stores/git', () => {
  const noop = async (): Promise<void> => {}
  return {
    useGitStore: () => ({
      availability: null,
      repositories: [],
      loadAvailability: noop,
      loadRepositories: noop,
    }),
  }
})

let failureCode: string | null = null
let wrapper: ReturnType<typeof mount> | null = null

const cases = [
  {
    state: 'confirmed disconnected',
    code: null,
    label: { en: 'No account connected', ru: 'Аккаунт не подключён' },
  },
  {
    state: 'credential storage failure',
    code: 'credential_storage',
    label: {
      en: 'Saved GitHub authorization unavailable',
      ru: 'Сохранённая авторизация GitHub недоступна',
    },
  },
  {
    state: 'auth failure',
    code: 'auth',
    label: {
      en: 'GitHub connection check failed',
      ru: 'Не удалось проверить подключение GitHub',
    },
  },
  {
    state: 'network failure',
    code: 'network',
    label: {
      en: 'GitHub connection check failed',
      ru: 'Не удалось проверить подключение GitHub',
    },
  },
  {
    state: 'internal failure',
    code: 'internal',
    label: {
      en: 'GitHub connection check failed',
      ru: 'Не удалось проверить подключение GitHub',
    },
  },
] as const

async function mountView(locale: 'en' | 'ru', code: string | null): Promise<void> {
  setActivePinia(createPinia())
  failureCode = code
  i18n.global.locale.value = locale
  setDevTransport(async <T,>(command: string) => {
    if (command === 'github_connection') {
      if (failureCode) {
        throw { code: failureCode, message: `connection failed: ${failureCode}` }
      }
      return null as T
    }
    throw { code: 'unsupported_capability', message: String(command) }
  })
  const router = createRouter({
    history: createMemoryHistory(),
    routes: [{ path: '/integrations', component: { template: '<div />' } }],
  })
  await router.push('/integrations')
  await router.isReady()
  wrapper = mount(IntegrationsView, {
    global: { plugins: [router, i18n] },
    attachTo: document.body,
  })
  await flushPromises()
  await flushPromises()
}

afterEach(() => {
  wrapper?.unmount()
  wrapper = null
  failureCode = null
  i18n.global.locale.value = 'en'
})

describe.each(['en', 'ru'] as const)('Integrations footer (%s)', (locale) => {
  for (const testCase of cases) {
    it(`${testCase.state} uses confirmed wording only when confirmed`, async () => {
      await mountView(locale, testCase.code)
      const text = wrapper?.text() ?? ''
      if (testCase.code === null) {
        expect(text).toContain(testCase.label[locale])
      } else {
        expect(text).not.toContain(cases[0].label[locale])
        expect(text).toContain(testCase.label[locale])
      }
    })
  }
})
