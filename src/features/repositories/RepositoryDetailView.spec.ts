import { flushPromises, mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { afterEach, describe, expect, it } from 'vitest'
import { createMemoryHistory, createRouter } from 'vue-router'

import { i18n } from '@/i18n'
import { setDevTransport } from '@/lib/ipc'
import type { GitOverview, Repository } from '@/types/ipc'
import RepositoryDetailView from './RepositoryDetailView.vue'

const timestamp = '2026-11-15T12:00:00.000Z'

function repository(withCheckout: boolean): Repository {
  return {
    id: 'repo-1',
    provider: 'github',
    host: 'github.com',
    externalId: '1',
    owner: 'owner',
    name: 'repo',
    fullName: 'owner/repo',
    description: null,
    visibility: 'public',
    defaultBranch: 'main',
    htmlUrl: 'https://github.com/owner/repo',
    cloneUrl: 'https://github.com/owner/repo.git',
    canonicalRemoteUrl: 'https://github.com/owner/repo',
    isArchived: false,
    identityConflict: null,
    checkouts: withCheckout
      ? [
          {
            id: 'checkout-1',
            repositoryId: 'repo-1',
            path: '/tmp/repo',
            lastCheckedAt: timestamp,
            createdAt: timestamp,
            updatedAt: timestamp,
          },
        ]
      : [],
    projects: [],
    lastCheckedAt: timestamp,
    createdAt: timestamp,
    updatedAt: timestamp,
  }
}

function overview(): GitOverview {
  return {
    repository: repository(true),
    status: {
      branch: 'main',
      upstream: null,
      ahead: 0,
      behind: 0,
      dirty: false,
      changes: [],
    },
    branches: [{ name: 'main', current: true }],
    log: [],
    diff: { filesChanged: 0, insertions: 0, deletions: 0, summary: '' },
  }
}

let wrapper: ReturnType<typeof mount> | null = null
let current: Repository
let removed = false
let detached = false
let router: ReturnType<typeof createRouter>

async function mountView(withCheckout: boolean): Promise<void> {
  setActivePinia(createPinia())
  current = repository(withCheckout)
  removed = false
  detached = false
  i18n.global.locale.value = 'en'
  setDevTransport(async <T,>(command: string) => {
    if (command === 'get_repository') return structuredClone(current) as T
    if (command === 'get_git_overview') return overview() as T
    if (command === 'remove_repository') {
      removed = true
      return undefined as T
    }
    if (command === 'detach_repository_checkout') {
      detached = true
      current = repository(false)
      return undefined as T
    }
    throw { code: 'unsupported_capability', message: String(command) }
  })
  router = createRouter({
    history: createMemoryHistory(),
    routes: [
      { path: '/repositories', component: { template: '<div />' } },
      { path: '/repositories/:id', component: { template: '<div />' } },
    ],
  })
  await router.push('/repositories/repo-1')
  await router.isReady()
  wrapper = mount(RepositoryDetailView, {
    global: { plugins: [router, i18n], directives: { tooltip: {} } },
    attachTo: document.body,
  })
  await flushPromises()
}

function button(text: string, modal = false): HTMLButtonElement | undefined {
  const root = modal ? document.querySelector('.fd-modal-panel') : document
  return Array.from(root?.querySelectorAll<HTMLButtonElement>('button') ?? []).find(
    (candidate) => candidate.textContent?.trim() === text,
  )
}

afterEach(() => {
  wrapper?.unmount()
  wrapper = null
  document.body.innerHTML = ''
})

describe('RepositoryDetailView management actions', () => {
  it('detaches only the checkout and leaves the repository', async () => {
    await mountView(true)
    expect(document.body.textContent).toContain('Local checkouts')
    button('Detach')?.click()
    await flushPromises()
    button('Detach', true)?.click()
    await flushPromises()
    expect(detached).toBe(true)
    expect(document.body.textContent).toContain('Remote only')
    expect(removed).toBe(false)
  })

  it('removes the repository after confirmation and returns to the list', async () => {
    await mountView(false)
    button('Remove repository')?.click()
    await flushPromises()
    button('Remove repository', true)?.click()
    await flushPromises()
    expect(removed).toBe(true)
    expect(router.currentRoute.value.path).toBe('/repositories')
  })
})
