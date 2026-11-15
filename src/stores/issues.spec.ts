import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it } from 'vitest'

import { handleIssueFixtureCommand } from '@/dev/issueFixtures'
import { setDevTransport } from '@/lib/ipc'
import { useIssueStore } from './issues'

describe('issue store', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    setDevTransport(async <T,>(command: Parameters<typeof handleIssueFixtureCommand>[0], args?: Record<string, unknown>) => {
      const result = handleIssueFixtureCommand<T>(command, args)
      if (!result.handled) {
        throw { code: 'unsupported_capability', message: String(command) }
      }
      return result.value as T
    })
  })

  it('loads workflow and issues', async () => {
    const store = useIssueStore()
    await store.load()
    expect(store.loading).toBe(false)
    expect(store.error).toBeNull()
    expect(store.statuses).toHaveLength(5)
    expect(store.issues.length).toBeGreaterThanOrEqual(12)
    expect(store.board).toHaveLength(5)
  })

  it('opens detail, updates, comments and archives', async () => {
    const store = useIssueStore()
    await store.load()
    await store.selectIssue('issue-101')
    expect(store.detail?.summary.key).toBe('FD-101')
    expect(store.detail?.comments.length).toBeGreaterThan(0)
    expect(store.detail?.relations.length).toBeGreaterThan(0)

    const updated = await store.updateIssue('issue-101', {
      priority: 'urgent',
      title: 'Improve GIF capture quality (edited)',
    })
    expect(updated).toBe(true)
    expect(store.detail?.summary.priority).toBe('urgent')
    expect(store.issues.find((issue) => issue.id === 'issue-101')?.title).toContain(
      'edited',
    )

    const commented = await store.addComment('issue-101', 'A local comment')
    expect(commented).toBe(true)
    expect(store.detail?.comments.some((comment) => comment.body === 'A local comment')).toBe(
      true,
    )

    const archived = await store.archiveIssue('issue-101', true)
    expect(archived).toBe(true)
    expect(store.selectedId).toBeNull()
    expect(store.issues.some((issue) => issue.id === 'issue-101')).toBe(false)
  })

  it('creates an issue and finds it with local search', async () => {
    const store = useIssueStore()
    await store.load()
    const created = await store.createIssue({
      title: 'New store issue',
      priority: 'high',
      issueType: 'feature',
      labelNames: ['store'],
    })
    expect(created.summary.key).toMatch(/^FD-\d+$/)
    expect(store.selectedId).toBe(created.summary.id)

    store.search = 'New store issue'
    await store.load()
    expect(store.issues).toHaveLength(1)
    expect(store.issues[0].title).toBe('New store issue')
  })

  it('updates several issues in one batch', async () => {
    const store = useIssueStore()
    await store.load()
    const ids = store.issues.slice(0, 2).map((issue) => issue.id)
    const ok = await store.batchUpdate(ids, { priority: 'low' })
    expect(ok).toBe(true)
    for (const id of ids) {
      expect(store.issues.find((issue) => issue.id === id)?.priority).toBe('low')
    }
  })
})
