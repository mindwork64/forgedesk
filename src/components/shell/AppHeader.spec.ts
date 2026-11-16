import { mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { afterEach, describe, expect, it } from 'vitest'
import { createMemoryHistory, createRouter } from 'vue-router'

import { i18n } from '@/i18n'
import AppHeader from './AppHeader.vue'

let wrapper: ReturnType<typeof mount> | null = null

async function mountHeader(locale: 'en' | 'ru'): Promise<void> {
  setActivePinia(createPinia())
  i18n.global.locale.value = locale
  const router = createRouter({
    history: createMemoryHistory(),
    routes: [{ path: '/', component: { template: '<div />' } }],
  })
  await router.push('/')
  await router.isReady()
  wrapper = mount(AppHeader, {
    global: {
      plugins: [router, i18n],
      directives: { tooltip: {} },
    },
  })
}

afterEach(() => {
  wrapper?.unmount()
  wrapper = null
  i18n.global.locale.value = 'en'
})

describe('AppHeader quick create', () => {
  it('is enabled, emits quick-create, and has the current EN tooltip', async () => {
    await mountHeader('en')
    const button = wrapper?.find('button[aria-label="Create issue"]')
    expect(button).toBeTruthy()
    expect(button?.attributes('disabled')).toBeUndefined()
    expect(i18n.global.te('header.quickCreateUnavailable')).toBe(false)
    await button?.trigger('click')
    expect(wrapper?.emitted('quick-create')).toHaveLength(1)
  })

  it('uses the current RU label and has no obsolete Stage 2 string', async () => {
    await mountHeader('ru')
    const button = wrapper?.find('button[aria-label="Создать задачу"]')
    expect(button).toBeTruthy()
    expect(i18n.global.te('header.quickCreateUnavailable')).toBe(false)
    expect(wrapper?.text()).not.toContain('этапе 2')
  })
})
