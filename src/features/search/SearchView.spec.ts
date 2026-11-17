import { flushPromises, mount } from '@vue/test-utils'
import { afterEach, describe, expect, it, vi } from 'vitest'
import { createMemoryHistory, createRouter } from 'vue-router'

import { i18n } from '@/i18n'
import { setDevTransport } from '@/lib/ipc'
import SearchView from './SearchView.vue'

const issue = {
  id: 'issue-1',
  key: 'FD-1',
  keyNumber: 1,
  title: 'Offline cache bug',
  status: { id: 'status-1', name: 'Todo', category: 'unstarted', color: 'neutral', position: 0, systemKey: 'todo' },
  priority: 'medium',
  issueType: 'bug',
  project: null,
  labels: [],
  estimate: null,
  startDate: null,
  dueDate: null,
  parentId: null,
  source: 'local',
  remoteUrl: null,
  archivedAt: null,
  createdAt: '2026-11-17T00:00:00.000Z',
  updatedAt: '2026-11-17T00:00:00.000Z',
  completedAt: null,
  subIssueCount: 0,
}

describe('SearchView', () => {
  let wrapper: ReturnType<typeof mount> | null = null
  let router: ReturnType<typeof createRouter>

  async function mountSearch(): Promise<void> {
    setDevTransport(async <T,>(command: string) => {
      if (command === 'list_initiatives' || command === 'list_documents' || command === 'list_repositories') {
        return [] as T
      }
      if (command === 'list_issues') return [issue] as T
      if (command === 'list_project_cards') return [] as T
      throw new Error(`Unexpected command: ${command}`)
    })
    router = createRouter({
      history: createMemoryHistory(),
      routes: [
        { path: '/search', component: SearchView },
        { path: '/my-issues', component: { template: '<div />' } },
        { path: '/projects/:id', component: { template: '<div />' } },
        { path: '/initiatives/:id', component: { template: '<div />' } },
        { path: '/documents', component: { template: '<div />' } },
        { path: '/repositories/:id', component: { template: '<div />' } },
      ],
    })
    await router.push('/search')
    await router.isReady()
    wrapper = mount(SearchView, { global: { plugins: [router, i18n] }, attachTo: document.body })
    await flushPromises()
  }

  afterEach(() => {
    wrapper?.unmount()
    wrapper = null
    document.body.innerHTML = ''
    vi.useRealTimers()
    i18n.global.locale.value = 'en'
  })

  it('searches the local issue index and opens a matching issue', async () => {
    vi.useFakeTimers()
    await mountSearch()
    await wrapper!.find('input').setValue('offline')
    await vi.advanceTimersByTimeAsync(200)
    await flushPromises()

    expect(wrapper!.text()).toContain('Offline cache bug')
    await wrapper!.find('button.fd-search-hit').trigger('click')
    await flushPromises()
    expect(router.currentRoute.value.path).toBe('/my-issues')
    expect(router.currentRoute.value.query.issue).toBe('issue-1')
  })

  it('does not run a search for a single character', async () => {
    vi.useFakeTimers()
    await mountSearch()
    await wrapper!.find('input').setValue('x')
    await vi.advanceTimersByTimeAsync(200)
    await flushPromises()
    expect(wrapper!.text()).toContain('Enter at least 2 characters')
    expect(wrapper!.text()).not.toContain('Offline cache bug')
  })
})
