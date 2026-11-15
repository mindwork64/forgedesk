import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it } from 'vitest'

import { setDevTransport } from '@/lib/ipc'
import type { CommandName, GitHubConnection } from '@/types/ipc'
import { useGithubAuthStore } from './githubAuth'

const connection: GitHubConnection = {
  connected: true,
  login: 'eloraam',
  userId: 1,
  scopes: '',
  expiresAt: '2026-11-15T20:00:00Z',
  refreshExpiresAt: '2026-11-30T20:00:00Z',
  storage: 'keyring',
  storageWarning: null,
}

let mode: 'error' | 'none' | 'connected' = 'none'

function installTransport(): void {
  setDevTransport(async <T,>(command: CommandName) => {
    switch (command) {
      case 'github_connection':
        if (mode === 'error') {
          throw { code: 'auth', message: 'Secure credential store could not read the GitHub token.' }
        }
        return (mode === 'connected' ? structuredClone(connection) : null) as T
      case 'github_start_device_flow':
        return {
          userCode: 'TEST-CODE',
          verificationUri: 'https://github.com/login/device',
          expiresIn: 899,
          interval: 5,
        } as T
      default:
        throw { code: 'unsupported_capability', message: String(command) }
    }
  })
}

describe('github auth store', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    mode = 'none'
    installTransport()
  })

  it('keeps absent credentials as disconnected', async () => {
    const store = useGithubAuthStore()
    await store.loadConnection()
    expect(store.connected).toBe(false)
    expect(store.connectionUnavailable).toBe(false)
    expect(store.error).toBeNull()
  })

  it('distinguishes keyring read failure from disconnected', async () => {
    mode = 'error'
    const store = useGithubAuthStore()
    await store.loadConnection()
    expect(store.connected).toBe(false)
    expect(store.connectionUnavailable).toBe(true)
    expect(store.error?.code).toBe('auth')
  })

  it('clears unavailable state after a successful connection', async () => {
    mode = 'error'
    const store = useGithubAuthStore()
    await store.loadConnection()
    expect(store.connectionUnavailable).toBe(true)

    mode = 'connected'
    await store.loadConnection()
    expect(store.connected).toBe(true)
    expect(store.connectionUnavailable).toBe(false)
    expect(store.error).toBeNull()
  })

  it('allows reauthorization while storage is unavailable', async () => {
    mode = 'error'
    const store = useGithubAuthStore()
    await store.loadConnection()
    await store.start()
    expect(store.flow?.userCode).toBe('TEST-CODE')
    expect(store.connectionUnavailable).toBe(true)
    store.cancel()
  })
})
