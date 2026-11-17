import { flushPromises, mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { afterEach, describe, expect, it } from 'vitest'
import { createMemoryHistory, createRouter } from 'vue-router'

import { i18n } from '@/i18n'
import { setDevTransport } from '@/lib/ipc'
import type { InboxEvent } from '@/types/ipc'
import ActivityView from './ActivityView.vue'

function activityEvent(): InboxEvent {
  return {
    id: 'event-1',
    source: 'github',
    kind: 'pr_merged',
    title: 'Pull request merged',
    summary: '#42 completed this issue',
    entityType: 'issue',
    entityId: 'issue-42',
    entityKey: 'FD-42',
    entityTitle: 'Ship activity timeline',
    createdAt: new Date().toISOString(),
    read: true,
    archived: true,
  }
}

describe('ActivityView', () => {
  let wrapper: ReturnType<typeof mount> | null = null
  let router: ReturnType<typeof createRouter>

  async function mountActivity(): Promise<void> {
    setActivePinia(createPinia())
    i18n.global.locale.value = 'en'
    setDevTransport(async <T,>(command: string) => {
      if (command === 'activity_list') return [activityEvent()] as T
      throw { code: 'unsupported_capability', message: command }
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
    wrapper = mount(ActivityView, { global: { plugins: [router, i18n] } })
    await flushPromises()
  }

  afterEach(() => {
    wrapper?.unmount()
    wrapper = null
    i18n.global.locale.value = 'en'
  })

  it('shows archived notification history in the independent activity feed', async () => {
    await mountActivity()
    expect(wrapper?.text()).toContain('Pull request merged')
    expect(wrapper?.text()).toContain('Archived in Inbox')
    expect(wrapper?.findAll('[data-testid="activity-event"]')).toHaveLength(1)
  })

  it('opens the linked issue from an activity entry', async () => {
    await mountActivity()
    await wrapper?.get('[data-testid="activity-event"]').trigger('click')
    await flushPromises()
    expect(router.currentRoute.value.path).toBe('/my-issues')
    expect(router.currentRoute.value.query.issue).toBe('issue-42')
  })
})
