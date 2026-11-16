import { flushPromises, mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { afterEach, describe, expect, it } from 'vitest'

import { i18n } from '@/i18n'
import { setDevTransport } from '@/lib/ipc'
import { useIssueStore } from '@/stores/issues'
import type { CreateIssueInput, IssueDetail } from '@/types/ipc'
import CreateIssueDialog from './CreateIssueDialog.vue'

const timestamp = '2026-11-15T12:00:00.000Z'
const status = {
  id: 'st-todo',
  name: 'Todo',
  category: 'unstarted' as const,
  color: 'info' as const,
  position: 0,
  systemKey: 'todo',
}

function detail(input: CreateIssueInput): IssueDetail {
  return {
    summary: {
      id: 'issue-1',
      key: 'FD-1',
      keyNumber: 1,
      title: input.title,
      status: { ...status },
      priority: input.priority ?? 'none',
      issueType: input.issueType ?? 'task',
      project: null,
      labels: [],
      estimate: input.estimate ?? null,
      startDate: null,
      dueDate: null,
      parentId: input.parentId ?? null,
      source: 'local',
      remoteUrl: null,
      archivedAt: null,
      createdAt: timestamp,
      updatedAt: timestamp,
      completedAt: null,
      subIssueCount: 0,
    },
    description: input.description ?? '',
    parent: null,
    children: [],
    relations: [],
    comments: [],
    activity: [],
  }
}

let wrapper: ReturnType<typeof mount> | null = null

async function mountDialog(parentId?: string): Promise<CreateIssueInput[]> {
  setActivePinia(createPinia())
  i18n.global.locale.value = 'en'
  const inputs: CreateIssueInput[] = []
  const issueStore = useIssueStore()
  issueStore.$patch({ workflow: { id: 'wf', name: 'Default', isDefault: true, statuses: [{ ...status, isDefault: true }] } })
  setDevTransport(async <T,>(command: string, args?: Record<string, unknown>) => {
    if (command === 'create_issue') {
      const input = args?.input as CreateIssueInput
      inputs.push(input)
      return detail(input) as T
    }
    if (command === 'list_workflow') {
      return { id: 'wf', name: 'Default', isDefault: true, statuses: [{ ...status, isDefault: true }] } as T
    }
    if (command === 'list_labels' || command === 'list_projects') {
      return [] as T
    }
    throw { code: 'unsupported_capability', message: String(command) }
  })
  wrapper = mount(CreateIssueDialog, {
    props: { modelValue: true, parentId: parentId ?? null, parentLabel: 'FD-9' },
    global: { plugins: [i18n], directives: { tooltip: {} } },
    attachTo: document.body,
  })
  await flushPromises()
  return inputs
}

afterEach(() => {
  wrapper?.unmount()
  wrapper = null
  document.body.innerHTML = ''
})

describe('CreateIssueDialog parent behavior', () => {
  it('passes parentId for sub-issue creation', async () => {
    const inputs = await mountDialog('parent-1')
    const input = document.querySelector<HTMLInputElement>('.fd-modal-panel input')!
    input.value = 'Sub-issue'
    input.dispatchEvent(new Event('input'))
    const create = Array.from(
      document.querySelectorAll<HTMLButtonElement>('.fd-modal-panel button'),
    ).find((button) => button.textContent?.includes('Create issue'))!
    create.click()
    await flushPromises()
    expect(inputs).toHaveLength(1)
    expect(inputs[0]?.parentId).toBe('parent-1')
    expect(inputs[0]?.title).toBe('Sub-issue')
  })
})
