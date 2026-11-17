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

let mode: 'none' | 'connected' = 'none'
let errorCode: string | null = null

function installTransport(): void {
  setDevTransport(async <T,>(command: CommandName) => {
    switch (command) {
      case 'github_connection':
        if (errorCode) {
          throw { code: errorCode, message: `connection failed: ${errorCode}` }
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
    errorCode = null
    installTransport()
  })

  it('keeps absent credentials as disconnected', async () => {
    const store = useGithubAuthStore()
    await store.loadConnection()
    expect(store.connected).toBe(false)
    expect(store.storageUnavailable).toBe(false)
    expect(store.connectionCheckFailed).toBe(false)
    expect(store.error).toBeNull()
  })

  it('shares a concurrent connection check instead of issuing duplicate requests', async () => {
    let resolveConnection!: (value: GitHubConnection | null) => void
    let requests = 0
    const pending = new Promise<GitHubConnection | null>((resolve) => {
      resolveConnection = resolve
    })
    setDevTransport(async <T,>(command: CommandName) => {
      if (command === 'github_connection') {
        requests += 1
        return pending as Promise<T>
      }
      if (command === 'selected_repository') return null as T
      throw { code: 'unsupported_capability', message: String(command) }
    })

    const store = useGithubAuthStore()
    const first = store.loadConnection()
    const second = store.loadConnection()
    expect(store.connectionLoading).toBe(true)
    expect(store.connectionChecked).toBe(false)

    resolveConnection(null)
    await Promise.all([first, second])

    expect(requests).toBe(1)
    expect(store.connectionLoading).toBe(false)
    expect(store.connectionChecked).toBe(true)
    expect(store.connected).toBe(false)
  })

  it('classifies credential storage failures as storage unavailable', async () => {
    errorCode = 'credential_storage'
    const store = useGithubAuthStore()
    await store.loadConnection()
    expect(store.connected).toBe(false)
    expect(store.storageUnavailable).toBe(true)
    expect(store.connectionCheckFailed).toBe(false)
    expect(store.error?.code).toBe('credential_storage')
  })

  it.each(['auth', 'network', 'internal'])(
    'keeps %s failures as a generic connection-check error',
    async (code) => {
      errorCode = code
      const store = useGithubAuthStore()
      await store.loadConnection()
      expect(store.connected).toBe(false)
      expect(store.storageUnavailable).toBe(false)
      expect(store.connectionCheckFailed).toBe(true)
      expect(store.error?.code).toBe(code)
    },
  )

  it('clears unavailable state after a successful connection', async () => {
    errorCode = 'credential_storage'
    const store = useGithubAuthStore()
    await store.loadConnection()
    expect(store.storageUnavailable).toBe(true)

    errorCode = null
    mode = 'connected'
    await store.loadConnection(true)
    expect(store.connected).toBe(true)
    expect(store.storageUnavailable).toBe(false)
    expect(store.connectionCheckFailed).toBe(false)
    expect(store.error).toBeNull()
  })

  it('allows reauthorization while storage is unavailable', async () => {
    errorCode = 'credential_storage'
    const store = useGithubAuthStore()
    await store.loadConnection()
    await store.start()
    expect(store.flow?.userCode).toBe('TEST-CODE')
    expect(store.storageUnavailable).toBe(true)
    store.cancel()
  })
})
