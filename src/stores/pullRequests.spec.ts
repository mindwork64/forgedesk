import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it } from 'vitest'

import { setDevTransport } from '@/lib/ipc'
import { usePullRequestStore } from '@/stores/pullRequests'

describe('pull request detail state', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  it('does not restore a detail pane when a pending request resolves after close', async () => {
    let resolveDetail!: (value: unknown) => void
    const pending = new Promise<unknown>((resolve) => {
      resolveDetail = resolve
    })
    setDevTransport(async <T,>() => pending as T)

    const store = usePullRequestStore()
    const opening = store.open('owner', 'repo', 42)

    expect(store.selectedNumber).toBe(42)
    expect(store.detailLoading).toBe(true)

    store.close()
    expect(store.selectedNumber).toBeNull()
    expect(store.detailLoading).toBe(false)

    resolveDetail(null)
    await opening

    expect(store.selectedNumber).toBeNull()
    expect(store.detail).toBeNull()
    expect(store.linkedIssue).toBeNull()
    expect(store.capabilities).toBeNull()
    expect(store.detailLoading).toBe(false)
  })
})
