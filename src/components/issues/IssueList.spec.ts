import { mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it } from 'vitest'

import { i18n } from '@/i18n'
import type { IssueSummary, StatusSummary } from '@/types/ipc'
import IssueList from './IssueList.vue'

const status: StatusSummary = {
  id: 'st-todo',
  name: 'Todo',
  category: 'unstarted',
  color: 'info',
  position: 1,
  systemKey: 'todo',
}

function issue(index: number): IssueSummary {
  return {
    id: `issue-${index}`,
    key: `FD-${index + 1}`,
    keyNumber: index + 1,
    title: `Virtual row ${index + 1}`,
    status,
    priority: 'medium',
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
    createdAt: '2026-11-15T12:00:00.000Z',
    updatedAt: '2026-11-15T12:00:00.000Z',
    completedAt: null,
    subIssueCount: 0,
  }
}

describe('IssueList virtualization', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    i18n.global.locale.value = 'en'
  })

  it('renders a window instead of every row above the threshold', async () => {
    const rows = Array.from({ length: 250 }, (_, index) => issue(index))
    const wrapper = mount(IssueList, {
      props: { issues: rows, selectedIds: [] },
      global: { plugins: [i18n] },
    })

    expect(wrapper.findAll('[data-testid="issue-row"]').length).toBeLessThan(30)
    expect(wrapper.findAll('[data-testid="issue-row"]').length).toBeGreaterThan(0)

    const body = wrapper.find('.fd-issue-list__body')
    ;(body.element as HTMLElement).scrollTop = 400
    await body.trigger('scroll')
    const visible = wrapper.findAll('[data-testid="issue-row"]')
    expect(visible.length).toBeGreaterThan(8)
    expect(visible.length).toBeLessThan(30)
    expect(visible[0]?.attributes('aria-label')).toBe('FD-3 Virtual row 3')
  })
})
