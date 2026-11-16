import { createPinia, setActivePinia } from 'pinia'
import { afterEach, beforeEach, describe, expect, it } from 'vitest'

import { setDevTransport } from '@/lib/ipc'
import { useGithubAuthStore } from '@/stores/githubAuth'
import type { Repository } from '@/types/ipc'

const repo: Repository = {
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
  checkouts: [],
  projects: [],
  lastCheckedAt: null,
  createdAt: '2026-11-15T12:00:00Z',
  updatedAt: '2026-11-15T12:00:00Z',
}

let selected: Repository | null = repo

beforeEach(() => {
  setActivePinia(createPinia())
  localStorage.clear()
  selected = repo
  setDevTransport(async <T,>(command: string, args?: Record<string, unknown>) => {
    if (command === 'github_connection') return null as T
    if (command === 'selected_repository') return structuredClone(selected) as T
    if (command === 'select_repository') {
      selected = args?.repositoryId ? repo : null
      return structuredClone(selected) as T
    }
    throw { code: 'unsupported_capability', message: String(command) }
  })
})

afterEach(() => {
  localStorage.clear()
})

describe('githubAuth repository selection', () => {
  it('restores the SQLite selection after connection load', async () => {
    const auth = useGithubAuthStore()
    await auth.loadConnection()
    expect(auth.selectedRepo).toEqual({
      repositoryId: 'repo-1',
      owner: 'owner',
      name: 'repo',
      fullName: 'owner/repo',
    })
  })

  it('clears a stale selection instead of using old owner/name', async () => {
    selected = null
    const auth = useGithubAuthStore()
    await auth.loadConnection()
    expect(auth.selectedRepo).toBeNull()
  })

  it('persists selection before returning', async () => {
    const auth = useGithubAuthStore()
    await auth.selectRepository(repo)
    expect(auth.selectedRepo?.repositoryId).toBe('repo-1')
    expect(selected?.id).toBe('repo-1')
  })
})
