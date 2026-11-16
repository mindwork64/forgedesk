import { flushPromises, mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { afterEach, describe, expect, it } from 'vitest'
import { createMemoryHistory, createRouter } from 'vue-router'

import { i18n } from '@/i18n'
import { setDevTransport } from '@/lib/ipc'
import { useAppStore } from '@/stores/app'
import type { AppSettings, Bootstrap, IssueDetail, Workspace } from '@/types/ipc'
import AppShell from './AppShell.vue'

const timestamp = '2026-11-15T12:00:00.000Z'
const workspace: Workspace = {
  id: 'ws-shell',
  name: 'Shell',
  keyPrefix: 'FD',
  createdAt: timestamp,
  updatedAt: timestamp,
}
const status = {
  id: 'st-todo',
  name: 'Todo',
  category: 'unstarted' as const,
  color: 'info' as const,
  position: 0,
  systemKey: 'todo',
}
const settings: AppSettings = {
  locale: 'en',
  theme: 'dark',
  sidebarCollapsed: false,
  lastRoute: '/my-issues',
  onboardingCompleted: true,
  detailPaneOpen: false,
  autoBackupEnabled: true,
  backupRetention: 20,
  window: { width: 1440, height: 900, x: null, y: null, maximized: false },
}
const bootstrap: Bootstrap = {
  appVersion: '0.1.0',
  schemaVersion: 10,
  dataDir: '/tmp/forgedesk',
  dbPath: '/tmp/forgedesk/forgedesk.db',
  backupsDir: '/tmp/forgedesk/backups',
  workspace,
  settings,
  backups: [],
  startupWarning: null,
}

function detail(title: string): IssueDetail {
  return {
    summary: {
      id: 'issue-1',
      key: 'FD-1',
      keyNumber: 1,
      title,
      status: {
        id: status.id,
        name: status.name,
        category: status.category,
        color: status.color,
        position: status.position,
        systemKey: status.systemKey,
      },
      priority: 'none',
      issueType: 'task',
      project: null,
      labels: [],
      estimate: null,
      startDate: null,
      dueDate: null,
      parentId: null,
      source: 'local',
      remoteUrl: null,
      archivedAt: null,
      createdAt: timestamp,
      updatedAt: timestamp,
      completedAt: null,
      subIssueCount: 0,
    },
    description: '',
    parent: null,
    children: [],
    relations: [],
    comments: [],
    activity: [],
  }
}

let wrapper: ReturnType<typeof mount> | null = null
let createCalls = 0
let workflowCalls = 0
let labelCalls = 0
let projectCalls = 0
let createError: Error | null = null
let createGate: Promise<void> | null = null

async function mountShell(hasWorkspace = true): Promise<void> {
  setActivePinia(createPinia())
  createCalls = 0
  workflowCalls = 0
  labelCalls = 0
  projectCalls = 0
  createError = null
  createGate = null
  i18n.global.locale.value = 'en'
  setDevTransport(async <T,>(command: string) => {
    switch (command) {
      case 'bootstrap':
        return structuredClone({ ...bootstrap, workspace: hasWorkspace ? workspace : null }) as T
      case 'list_workflow':
        workflowCalls += 1
        return {
          id: 'wf',
          name: 'Default',
          isDefault: true,
          statuses: [{ ...status, isDefault: true }],
        } as T
      case 'list_labels':
        labelCalls += 1
        return [] as T
      case 'list_projects':
        projectCalls += 1
        return [] as T
      case 'save_settings':
        return structuredClone(settings) as T
      case 'create_issue':
        createCalls += 1
        if (createGate) await createGate
        if (createError) throw createError
        return detail('Quick task') as T
      default:
        throw { code: 'unsupported_capability', message: String(command) }
    }
  })
  const app = useAppStore()
  await app.load()
  const router = createRouter({
    history: createMemoryHistory(),
    routes: [
      { path: '/my-issues', component: { template: '<div />' } },
      { path: '/repositories', component: { template: '<div />' } },
      { path: '/:pathMatch(.*)*', component: { template: '<div />' } },
    ],
  })
  await router.push('/repositories')
  await router.isReady()
  wrapper = mount(AppShell, {
    global: {
      plugins: [router, i18n],
      stubs: {
        AppSidebar: true,
        StatusStrip: true,
        CommandPalette: true,
        ShortcutsDialog: true,
        RouterView: true,
      },
      directives: { tooltip: {} },
    },
    attachTo: document.body,
  })
  await flushPromises()
}

function quickCreateButton(): HTMLButtonElement {
  return document.querySelector<HTMLButtonElement>(
    'button[aria-label="Create issue"]',
  )!
}

function modalTitleInput(): HTMLInputElement {
  return document.querySelector<HTMLInputElement>('.fd-modal-panel input')!
}

function modalCreateButton(): HTMLButtonElement {
  const buttons = Array.from(
    document.querySelectorAll<HTMLButtonElement>('.fd-modal-panel button'),
  )
  return buttons.find((button) => button.textContent?.includes('Create issue'))!
}

afterEach(() => {
  wrapper?.unmount()
  wrapper = null
  document.body.innerHTML = ''
})

describe('AppShell global quick create', () => {
  it('opens the existing dialog from another section, loads references, and creates', async () => {
    await mountShell()
    expect(workflowCalls).toBe(0)
    quickCreateButton().click()
    await flushPromises()
    expect(workflowCalls).toBe(1)
    expect(labelCalls).toBe(1)
    expect(projectCalls).toBe(1)
    expect(document.body.textContent).toContain('New issue')

    modalTitleInput().value = 'Quick task'
    modalTitleInput().dispatchEvent(new Event('input'))
    modalCreateButton().click()
    await flushPromises()
    expect(createCalls).toBe(1)
    expect(document.querySelector('.fd-modal-panel')).toBeNull()
  })

  it('keeps create errors visible in the global dialog', async () => {
    await mountShell()
    createError = new Error('create failed')
    quickCreateButton().click()
    await flushPromises()
    modalTitleInput().value = 'Will fail'
    modalTitleInput().dispatchEvent(new Event('input'))
    modalCreateButton().click()
    await flushPromises()
    expect(createCalls).toBe(1)
    expect(document.querySelector('.fd-modal-panel')).not.toBeNull()
    expect(document.querySelector('.fd-issue-create__error')?.textContent).toContain(
      'create failed',
    )
  })

  it('does not duplicate creation on a double submit', async () => {
    await mountShell()
    let release!: () => void
    createGate = new Promise<void>((resolve) => {
      release = resolve
    })
    quickCreateButton().click()
    await flushPromises()
    modalTitleInput().value = 'Only once'
    modalTitleInput().dispatchEvent(new Event('input'))
    modalCreateButton().click()
    modalCreateButton().click()
    await flushPromises()
    expect(createCalls).toBe(1)
    release()
    await flushPromises()
  })

  it('does nothing when no workspace exists', async () => {
    await mountShell(false)
    quickCreateButton().click()
    await flushPromises()
    expect(workflowCalls).toBe(0)
    expect(document.querySelector('.fd-modal-panel')).toBeNull()
  })
})
