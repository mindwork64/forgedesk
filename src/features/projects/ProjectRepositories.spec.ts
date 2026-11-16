import { flushPromises, mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { afterEach, describe, expect, it } from 'vitest'
import { createMemoryHistory, createRouter } from 'vue-router'

import { i18n } from '@/i18n'
import { setDevTransport } from '@/lib/ipc'
import type { Repository } from '@/types/ipc'
import ProjectRepositories from './ProjectRepositories.vue'

const timestamp = '2026-11-15T12:00:00.000Z'

function repo(id: string, fullName: string, projects: string[]): Repository {
  return {
    id,
    provider: 'github',
    host: 'github.com',
    externalId: id,
    owner: fullName.split('/')[0]!,
    name: fullName.split('/')[1]!,
    fullName,
    description: null,
    visibility: 'public',
    defaultBranch: 'main',
    htmlUrl: `https://github.com/${fullName}`,
    cloneUrl: `https://github.com/${fullName}.git`,
    canonicalRemoteUrl: `https://github.com/${fullName}`,
    isArchived: false,
    identityConflict: null,
    checkouts: [],
    projects: projects.map((id) => ({ id, name: id })),
    lastCheckedAt: null,
    createdAt: timestamp,
    updatedAt: timestamp,
  }
}

let wrapper: ReturnType<typeof mount> | null = null
let linked: Repository[] = []
let all: Repository[] = []
let unlinkCalls = 0

async function mountView(): Promise<void> {
  setActivePinia(createPinia())
  i18n.global.locale.value = 'en'
  unlinkCalls = 0
  setDevTransport(async <T,>(command: string, args?: Record<string, unknown>) => {
    if (command === 'list_repositories') return structuredClone(all) as T
    if (command === 'get_project_repositories') return structuredClone(linked) as T
    if (command === 'link_repository') {
      const found = all.find((item) => item.id === args?.repositoryId)!
      const updated = { ...found, projects: [{ id: String(args?.projectId), name: 'Project' }] }
      linked = [...linked, updated]
      return structuredClone(updated) as T
    }
    if (command === 'unlink_repository') {
      unlinkCalls += 1
      const found = linked.find((item) => item.id === args?.repositoryId)!
      linked = linked.filter((item) => item.id !== found.id)
      return structuredClone({ ...found, projects: [] }) as T
    }
    throw { code: 'unsupported_capability', message: String(command) }
  })
  const router = createRouter({
    history: createMemoryHistory(),
    routes: [
      { path: '/projects/:id', component: { template: '<div />' } },
      { path: '/repositories', component: { template: '<div />' } },
      { path: '/repositories/:id', component: { template: '<div />' } },
      { path: '/integrations', component: { template: '<div />' } },
    ],
  })
  await router.push('/projects/p1')
  await router.isReady()
  wrapper = mount(ProjectRepositories, {
    props: { projectId: 'p1' },
    global: { plugins: [router, i18n], directives: { tooltip: {} } },
    attachTo: document.body,
  })
  await flushPromises()
}

afterEach(() => {
  wrapper?.unmount()
  wrapper = null
  document.body.innerHTML = ''
})

describe('ProjectRepositories', () => {
  it('shows linked repositories and remote-only state', async () => {
    linked = [repo('r1', 'owner/one', ['p1'])]
    all = [linked[0]!]
    await mountView()
    expect(document.body.textContent).toContain('owner/one')
    expect(document.body.textContent).toContain('Remote only')
  })

  it('shows empty state and links an existing repository', async () => {
    all = [repo('r1', 'owner/one', [])]
    linked = []
    await mountView()
    expect(document.body.textContent).toContain('No repositories linked')
    const select = document.querySelector<HTMLSelectElement>('.fd-project-repository-link select')
    if (!select) throw new Error('select missing')
    select.value = 'r1'
    select.dispatchEvent(new Event('change'))
    await flushPromises()
    const link = Array.from(document.querySelectorAll<HTMLButtonElement>('button')).find(
      (button) => button.textContent?.includes('Link repository'),
    )
    link?.click()
    await flushPromises()
    expect(document.body.textContent).toContain('owner/one')
    expect(document.body.textContent).not.toContain('No repositories linked')
  })

  it('reloads repositories when the project id changes without remount', async () => {
    linked = [repo('r1', 'owner/one', ['p1'])]
    all = [linked[0]!]
    await mountView()
    expect(document.body.textContent).toContain('owner/one')

    linked = [repo('r2', 'owner/two', ['p2'])]
    all = [linked[0]!]
    await wrapper?.setProps({ projectId: 'p2' })
    await flushPromises()
    expect(document.body.textContent).toContain('owner/two')
    expect(document.body.textContent).not.toContain('owner/one')
  })

  it('unlinks only the association', async () => {
    linked = [repo('r1', 'owner/one', ['p1'])]
    all = [linked[0]!]
    await mountView()
    const unlink = Array.from(document.querySelectorAll<HTMLButtonElement>('button')).find(
      (button) => button.textContent?.trim() === 'Unlink',
    )
    unlink?.click()
    await flushPromises()
    const confirm = Array.from(document.querySelectorAll<HTMLButtonElement>('.fd-modal-panel button')).find(
      (button) => button.textContent?.trim() === 'Unlink',
    )
    confirm?.click()
    await flushPromises()
    expect(unlinkCalls).toBe(1)
    expect(document.body.textContent).toContain('No repositories linked')
    expect(all).toHaveLength(1)
  })
})
