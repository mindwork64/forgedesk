import { flushPromises, mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { afterEach, beforeEach, describe, expect, it } from 'vitest'
import { createMemoryHistory, createRouter } from 'vue-router'

import { i18n } from '@/i18n'
import { setDevTransport } from '@/lib/ipc'
import { useGithubAuthStore } from '@/stores/githubAuth'
import { useToastStore } from '@/stores/toasts'
import type { GitHubImportResult, Repository } from '@/types/ipc'
import GitHubRepoDialog from './GitHubRepoDialog.vue'

const repo: Repository = {
  id: 'repo-1',
  provider: 'github',
  host: 'github.com',
  externalId: '123',
  owner: 'mindwork64',
  name: 'forgedesk',
  fullName: 'mindwork64/forgedesk',
  description: 'Test repository',
  visibility: 'public',
  defaultBranch: 'main',
  htmlUrl: 'https://github.com/mindwork64/forgedesk',
  cloneUrl: 'https://github.com/mindwork64/forgedesk.git',
  canonicalRemoteUrl: 'https://github.com/mindwork64/forgedesk',
  isArchived: false,
  identityConflict: null,
  checkouts: [],
  projects: [],
  lastCheckedAt: null,
  createdAt: '2026-11-15T12:00:00Z',
  updatedAt: '2026-11-15T12:00:00Z',
}
const result: GitHubImportResult = {
  imported: 2,
  skipped: 1,
  projectId: 'project-1',
  projectName: 'mindwork64/forgedesk',
  repositoryId: 'repo-1',
  hasMore: true,
  pages: 5,
  nextPage: 6,
}

let wrapper: ReturnType<typeof mount> | null = null
let currentResult: GitHubImportResult = structuredClone(result)
let discoveryProjects: { id: string; name: string }[] = []
let importCalls = 0
let importPages: number[] = []
let lastTarget: string | null = null
let importError: Error | null = null
let importGate: Promise<void> | null = null
let router: ReturnType<typeof createRouter>

async function mountDialog(projects: { id: string; name: string }[] = []): Promise<void> {
  setActivePinia(createPinia())
  importCalls = 0
  currentResult = structuredClone(result)
  discoveryProjects = projects
  importPages = []
  lastTarget = null
  importError = null
  importGate = null
  i18n.global.locale.value = 'en'
  localStorage.clear()
  const discovered = () => ({ ...repo, projects: discoveryProjects })
  setDevTransport(async <T,>(command: string, args?: Record<string, unknown>) => {
    if (command === 'github_discover_repositories') return { items: [discovered()], hasMore: false, pagesLoaded: 1, nextPage: null } as T
    if (command === 'select_repository') return structuredClone(discovered()) as T
    if (command === 'github_import_issues') {
      importCalls += 1
      importPages.push(Number(args?.startPage ?? 1))
      lastTarget = (args?.targetProjectId as string | null | undefined) ?? null
      if (importGate) await importGate
      if (importError) throw importError
      return structuredClone(currentResult) as T
    }
    throw { code: 'unsupported_capability', message: String(command) }
  })
  const auth = useGithubAuthStore()
  auth.repositories = [discovered()]
  router = createRouter({
    history: createMemoryHistory(),
    routes: [
      { path: '/pull-requests', component: { template: '<div />' } },
      { path: '/projects/:id', component: { template: '<div />' } },
    ],
  })
  await router.push('/pull-requests')
  await router.isReady()
  wrapper = mount(GitHubRepoDialog, {
    props: { open: true },
    global: { plugins: [router, i18n], directives: { tooltip: {} } },
    attachTo: document.body,
  })
  await flushPromises()
}

function button(text: string): HTMLButtonElement {
  const match = Array.from(
    document.querySelectorAll<HTMLButtonElement>('.fd-modal-panel button'),
  ).find((candidate) => candidate.textContent?.includes(text))
  if (!match) throw new Error(`button not found: ${text}`)
  return match
}

afterEach(() => {
  wrapper?.unmount()
  wrapper = null
  document.body.innerHTML = ''
  localStorage.clear()
})

describe('GitHubRepoDialog', () => {
  beforeEach(async () => {
    await mountDialog()
  })

  it('renders camelCase repository fields, public state and description', async () => {
    const text = document.body.textContent ?? ''
    expect(text).toContain('mindwork64/forgedesk')
    expect(text).toContain('main')
    expect(text).toContain('Public')
    expect(text).toContain('Test repository')

    i18n.global.locale.value = 'ru'
    await flushPromises()
    expect(document.body.textContent).toContain('Публичный')
    expect(document.body.textContent).toContain('mindwork64/forgedesk')
  })

  it('searches by fullName and description', async () => {
    const input = document.querySelector<HTMLInputElement>(
      '.fd-modal-panel input[type="search"]',
    )!
    input.value = 'mindwork64/forgedesk'
    input.dispatchEvent(new Event('input'))
    await flushPromises()
    expect(document.body.textContent).toContain('mindwork64/forgedesk')

    input.value = 'Test repository'
    input.dispatchEvent(new Event('input'))
    await flushPromises()
    expect(document.body.textContent).toContain('mindwork64/forgedesk')
  })

  it('selects owner/name for pull requests and navigates', async () => {
    button('Use for pull requests').click()
    await flushPromises()
    const auth = useGithubAuthStore()
    expect(auth.selectedRepo).toEqual({
      repositoryId: 'repo-1',
      owner: 'mindwork64',
      name: 'forgedesk',
      fullName: 'mindwork64/forgedesk',
    })
    expect(router.currentRoute.value.path).toBe('/pull-requests')
  })

  it('imports once, shows the result, and opens the project', async () => {
    importGate = new Promise<void>((resolve) => {
      setTimeout(resolve, 10)
    })
    button('Import issues').click()
    button('Import issues').click()
    await flushPromises()
    expect(importCalls).toBe(1)
    await new Promise((resolve) => setTimeout(resolve, 15))
    await flushPromises()
    expect(document.body.textContent).toContain('Imported 2, skipped 1')
    expect(document.body.textContent).toContain('more issues remain')
    expect(document.body.textContent).toContain('Continue import')
    currentResult = { ...result, hasMore: false, nextPage: null }
    button('Continue import').click()
    await flushPromises()
    expect(importCalls).toBe(2)
    expect(importPages[1]).toBe(6)
    expect(document.body.textContent).not.toContain('Continue import')
    button('View imported project').click()
    await flushPromises()
    expect(router.currentRoute.value.path).toBe('/projects/project-1')
  })

  it('requires an explicit target project when multiple projects are linked', async () => {
    await wrapper?.unmount()
    wrapper = null
    document.body.innerHTML = ''
    await mountDialog([
      { id: 'p1', name: 'One' },
      { id: 'p2', name: 'Two' },
    ])
    const select = document.querySelector<HTMLSelectElement>('.fd-github-repo-target')!
    const importButton = button('Import issues')
    expect(importButton.disabled).toBe(true)
    select.value = 'p2'
    select.dispatchEvent(new Event('change'))
    await flushPromises()
    expect(importButton.disabled).toBe(false)
    importButton.click()
    await flushPromises()
    expect(lastTarget).toBe('p2')
  })

  it('drops a stale target when discovery no longer returns it', async () => {
    await wrapper?.unmount()
    wrapper = null
    document.body.innerHTML = ''
    await mountDialog([
      { id: 'p1', name: 'One' },
      { id: 'p2', name: 'Two' },
    ])
    const select = document.querySelector<HTMLSelectElement>('.fd-github-repo-target')!
    select.value = 'p1'
    select.dispatchEvent(new Event('change'))
    await flushPromises()
    const auth = useGithubAuthStore()
    auth.repositories = [{ ...repo, projects: [{ id: 'p2', name: 'Two' }] }]
    await flushPromises()
    button('Import issues').click()
    await flushPromises()
    expect(lastTarget).toBeNull()
  })

  it('clears a selected target when the dialog is reopened', async () => {
    await wrapper?.unmount()
    wrapper = null
    document.body.innerHTML = ''
    await mountDialog([
      { id: 'p1', name: 'One' },
      { id: 'p2', name: 'Two' },
    ])
    const select = document.querySelector<HTMLSelectElement>('.fd-github-repo-target')!
    select.value = 'p1'
    select.dispatchEvent(new Event('change'))
    await flushPromises()
    await wrapper?.setProps({ open: false })
    discoveryProjects = [{ id: 'p2', name: 'Two' }]
    await wrapper?.setProps({ open: true })
    await flushPromises()
    button('Import issues').click()
    await flushPromises()
    expect(lastTarget).toBeNull()
  })

  it('keeps the import error visible through the existing toast path', async () => {
    importError = new Error('import failed')
    button('Import issues').click()
    await flushPromises()
    const toasts = useToastStore().items
    expect(toasts.some((toast) => toast.kind === 'error')).toBe(true)
  })
})
