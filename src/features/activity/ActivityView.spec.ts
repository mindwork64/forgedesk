import { flushPromises, mount } from '@vue/test-utils'
import { afterEach, describe, expect, it } from 'vitest'
import { createMemoryHistory, createRouter } from 'vue-router'

import { i18n } from '@/i18n'
import { setDevTransport } from '@/lib/ipc'
import ActivityView from './ActivityView.vue'

const event = {
  id: 'event-1',
  issueId: 'issue-1',
  issueKey: 'FD-1',
  issueTitle: 'Offline cache bug',
  source: 'local',
  kind: 'issue_created',
  data: { key: 'FD-1' },
  createdAt: '2026-11-17T00:00:00.000Z',
}

describe('ActivityView', () => {
  let wrapper: ReturnType<typeof mount> | null = null
  let router: ReturnType<typeof createRouter>

  afterEach(() => {
    wrapper?.unmount()
    wrapper = null
    document.body.innerHTML = ''
    i18n.global.locale.value = 'en'
  })

  it('renders the workspace activity feed and opens the related issue', async () => {
    setDevTransport(async <T,>(command: string) => {
      if (command === 'activity_list') return [event] as T
      throw new Error(`Unexpected command: ${command}`)
    })
    router = createRouter({
      history: createMemoryHistory(),
      routes: [
        { path: '/activity', component: ActivityView },
        { path: '/my-issues', component: { template: '<div />' } },
      ],
    })
    await router.push('/activity')
    await router.isReady()
    wrapper = mount(ActivityView, { global: { plugins: [router, i18n] }, attachTo: document.body })
    await flushPromises()

    expect(wrapper.text()).toContain('Issue created')
    expect(wrapper.text()).toContain('FD-1 · Offline cache bug')
    await wrapper.find('button.fd-activity-row').trigger('click')
    await flushPromises()
    expect(router.currentRoute.value.path).toBe('/my-issues')
    expect(router.currentRoute.value.query.issue).toBe('issue-1')
  })
})
