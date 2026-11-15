import { defineStore } from 'pinia'
import { computed, ref } from 'vue'

import { i18n } from '@/i18n'
import { api, toIpcError } from '@/lib/ipc'
import { useToastStore } from '@/stores/toasts'
import type {
  DeviceFlowStart,
  GitHubConnection,
  GitHubImportResult,
  GitHubRepo,
} from '@/types/ipc'

export const useGithubAuthStore = defineStore('githubAuth', () => {
  const t = i18n.global.t
  const toast = useToastStore()

  const connection = ref<GitHubConnection | null>(null)
  const flow = ref<DeviceFlowStart | null>(null)
  const polling = ref(false)
  const starting = ref(false)
  const error = ref<ReturnType<typeof toIpcError> | null>(null)
  const repositories = ref<GitHubRepo[]>([])
  const selectedRepo = ref<{ owner: string; name: string; fullName: string } | null>(
    readSelectedRepo(),
  )
  const discovering = ref(false)
  const importing = ref(false)
  let timer: ReturnType<typeof setTimeout> | null = null

  const connected = computed(() => connection.value?.connected === true)
  const storageUnavailable = ref(false)
  const connectionUnavailable = computed(
    () => !connected.value && storageUnavailable.value,
  )

  function readSelectedRepo(): { owner: string; name: string; fullName: string } | null {
    try {
      const raw = localStorage.getItem('forgedesk.githubRepo')
      return raw ? JSON.parse(raw) : null
    } catch {
      return null
    }
  }

  function selectRepository(repo: GitHubRepo): void {
    selectedRepo.value = {
      owner: repo.owner.login,
      name: repo.name,
      fullName: repo.fullName,
    }
    try {
      localStorage.setItem('forgedesk.githubRepo', JSON.stringify(selectedRepo.value))
    } catch {
      // Storage is optional; the in-memory selection still works.
    }
  }

  function clearTimer(): void {
    if (timer) {
      clearTimeout(timer)
      timer = null
    }
  }

  async function loadConnection(): Promise<void> {
    try {
      connection.value = await api.githubConnection()
      error.value = null
      storageUnavailable.value = false
    } catch (caught) {
      error.value = toIpcError(caught)
      storageUnavailable.value = true
    }
  }

  async function start(): Promise<void> {
    starting.value = true
    try {
      flow.value = await api.githubStartDeviceFlow()
      polling.value = true
      schedulePoll(flow.value.interval)
    } catch (caught) {
      error.value = toIpcError(caught)
      toast.error(t('githubAuth.startError'), error.value.message, error.value.detail)
    } finally {
      starting.value = false
    }
  }

  function schedulePoll(seconds: number): void {
    clearTimer()
    timer = setTimeout(() => {
      void poll()
    }, Math.max(5, seconds) * 1000)
  }

  async function poll(): Promise<void> {
    if (!flow.value) {
      return
    }
    try {
      const result = await api.githubPollDeviceFlow()
      if (result.connection) {
        connection.value = result.connection
        error.value = null
        storageUnavailable.value = false
        flow.value = null
        polling.value = false
        clearTimer()
        if (result.connection.storage === 'memory') {
          toast.warning(
            t('githubAuth.connected'),
            result.connection.storageWarning ?? t('githubAuth.memoryOnly'),
          )
        } else {
          toast.success(t('githubAuth.connected'), result.connection.login)
        }
        return
      }
      schedulePoll(result.interval)
    } catch (caught) {
      const wire = toIpcError(caught)
      error.value = wire
      flow.value = null
      polling.value = false
      clearTimer()
      toast.error(t('githubAuth.authError'), wire.message, wire.detail)
    }
  }

  function cancel(): void {
    flow.value = null
    polling.value = false
    clearTimer()
  }

  async function refresh(): Promise<void> {
    try {
      connection.value = await api.githubRefresh()
      toast.success(t('githubAuth.refreshed'), connection.value.login)
    } catch (caught) {
      const wire = toIpcError(caught)
      toast.error(t('githubAuth.refreshError'), wire.message, wire.detail)
    }
  }

  async function discoverRepositories(): Promise<void> {
    discovering.value = true
    try {
      repositories.value = await api.githubDiscoverRepositories()
    } catch (caught) {
      const wire = toIpcError(caught)
      toast.error(t('githubRepos.discoverError'), wire.message, wire.detail)
    } finally {
      discovering.value = false
    }
  }

  async function importIssues(
    owner: string,
    repo: string,
  ): Promise<GitHubImportResult | null> {
    importing.value = true
    try {
      const result = await api.githubImportIssues(owner, repo)
      toast.success(
        t('githubRepos.imported', {
          imported: result.imported,
          skipped: result.skipped,
        }),
        result.projectName,
      )
      return result
    } catch (caught) {
      const wire = toIpcError(caught)
      toast.error(t('githubRepos.importError'), wire.message, wire.detail)
      return null
    } finally {
      importing.value = false
    }
  }

  async function disconnect(): Promise<void> {
    try {
      await api.githubDisconnect()
      connection.value = null
      error.value = null
      storageUnavailable.value = false
      toast.success(t('githubAuth.disconnected'))
    } catch (caught) {
      const wire = toIpcError(caught)
      toast.error(t('githubAuth.disconnectError'), wire.message, wire.detail)
    }
  }

  return {
    connection,
    flow,
    polling,
    starting,
    error,
    connected,
    connectionUnavailable,
    repositories,
    selectedRepo,
    discovering,
    importing,
    loadConnection,
    start,
    poll,
    cancel,
    refresh,
    discoverRepositories,
    importIssues,
    selectRepository,
    disconnect,
  }
})
