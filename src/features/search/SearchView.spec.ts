import { flushPromises, mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { afterEach, describe, expect, it } from 'vitest'
import { createMemoryHistory, createRouter } from 'vue-router'

import { i18n } from '@/i18n'
import { setDevTransport } from '@/lib/ipc'
import SearchView from './SearchView.vue'

describe('SearchView', () => {
  let wrapper: ReturnType<typeof mount> | null = null
  let router: ReturnType<typeof createRouter>

  async function mountSearch(): Promise<void> {
    setActivePinia(createPinia())
    i18n.global.locale.value = 'en'
    setDevTransport(async <T,>(command: string, args?: Record<string, unknown>) => {
      switch (command) {
        case 'list_initiatives':
          return [{
            id: 'initiative-1', name: 'Launch', description: 'Reliability programme',
            status: 'active', priority: 'high', startDate: null, targetDate: null,
            progress: { percent: 0, completed: 0, total: 0 }, projectCount: 0,
            createdAt: '', updatedAt: '', archivedAt: null,
          }] as T
        case 'list_documents':
          return [{
            id: 'doc-1', title: 'ForgeDesk notes', body: 'Design for ForgeDesk search',
            projectId: null, initiativeId: null, projectName: null, initiativeName: null,
            archivedAt: null, createdAt: '', updatedAt: '',
          }] as unknown as T
        case 'list_repositories':
          return [{
            id: 'repo-1', provider: 'github', host: 'github.com', externalId: '1',
            owner: 'mindwork64', name: 'forgedesk', fullName: 'mindwork64/forgedesk',
            description: 'ForgeDesk source', visibility: 'public', defaultBranch: 'main',
            htmlUrl: null, cloneUrl: null, canonicalRemoteUrl: null, isArchived: false,
            identityConflict: null, checkouts: [], projects: [], lastCheckedAt: null,
            createdAt: '', updatedAt: '',
          }] as T
        case 'list_issues':
          return [] as unknown as T
        case 'list_project_cards': {
          const query = args?.query as { search?: string } | undefined
          if (!(query?.search ?? '').toLowerCase().includes('forgedesk')) return [] as T
          return [{
            id: 'project-1', name: 'ForgeDesk project', description: 'Workspace',
            status: 'active', priority: 'medium', startDate: null, targetDate: null,
            initiative: null, progress: { percent: 0, completed: 0, total: 0 },
            color: 'blue', createdAt: '', updatedAt: '', archivedAt: null,
          }] as T
        }
        default:
          throw { code: 'unsupported_capability', message: command }
      }
    })
    router = createRouter({
      history: createMemoryHistory(),
      routes: [
        { path: '/search', component: SearchView },
        { path: '/projects/:id', component: { template: '<div />' } },
        { path: '/initiatives/:id', component: { template: '<div />' } },
        { path: '/documents', component: { template: '<div />' } },
        { path: '/repositories/:id', component: { template: '<div />' } },
        { path: '/my-issues', component: { template: '<div />' } },
      ],
    })
    await router.push('/search')
    await router.isReady()
    wrapper = mount(SearchView, { global: { plugins: [router, i18n] } })
    await flushPromises()
  }

  afterEach(() => {
    wrapper?.unmount()
    wrapper = null
    i18n.global.locale.value = 'en'
  })

  it('searches projects, documents and repositories in the local workspace', async () => {
    await mountSearch()
    await wrapper?.get('input').setValue('ForgeDesk')
    await new Promise((resolve) => setTimeout(resolve, 220))
    await flushPromises()

    expect(wrapper?.findAll('[data-testid^="search-result-"]')).toHaveLength(3)
    expect(wrapper?.text()).toContain('ForgeDesk project')
    expect(wrapper?.text()).toContain('ForgeDesk notes')
    expect(wrapper?.text()).toContain('mindwork64/forgedesk')
  })

  it('searches initiative names and descriptions', async () => {
    await mountSearch()
    await wrapper?.get('input').setValue('Reliability')
    await new Promise((resolve) => setTimeout(resolve, 220))
    await flushPromises()
    expect(wrapper?.findAll('[data-testid^="search-result-"]')).toHaveLength(1)
    expect(wrapper?.get('[data-testid="search-result-initiative"]').text()).toContain('Launch')
  })

  it('navigates to a selected initiative', async () => {
    await mountSearch()
    await wrapper?.get('input').setValue('Reliability')
    await new Promise((resolve) => setTimeout(resolve, 220))
    await flushPromises()
    await wrapper?.get('[data-testid="search-result-initiative"]').trigger('click')
    await flushPromises()
    expect(router.currentRoute.value.path).toBe('/initiatives/initiative-1')
  })

  it('navigates to the selected project', async () => {
    await mountSearch()
    await wrapper?.get('input').setValue('ForgeDesk')
    await new Promise((resolve) => setTimeout(resolve, 220))
    await flushPromises()
    await wrapper?.get('[data-testid="search-result-project"]').trigger('click')
    await flushPromises()
    expect(router.currentRoute.value.path).toBe('/projects/project-1')
  })
})
