import { defineStore } from 'pinia'
import { computed, ref } from 'vue'

import { i18n } from '@/i18n'
import { api, toIpcError } from '@/lib/ipc'
import { useToastStore } from '@/stores/toasts'
import type {
  DeviceFlowStart,
  GitHubConnection,
  GitHubImportResult,
  Repository,
} from '@/types/ipc'

interface SelectedRepository {
  repositoryId: string
  owner: string
  name: string
  fullName: string
}

export const useGithubAuthStore = defineStore('githubAuth', () => {
  const t = i18n.global.t
  const toast = useToastStore()

  const connection = ref<GitHubConnection | null>(null)
  const flow = ref<DeviceFlowStart | null>(null)
  const polling = ref(false)
  const starting = ref(false)
  const error = ref<ReturnType<typeof toIpcError> | null>(null)
  const repositories = ref<Repository[]>([])
  const selectedRepo = ref<SelectedRepository | null>(readSelectedRepo())
  const discoveryHasMore = ref(false)
  const discoveryPages = ref(0)
  const discoveryNextPage = ref<number | null>(null)
  const discovering = ref(false)
  const importing = ref(false)
  const syncingOutbox = ref(false)
  let timer: ReturnType<typeof setTimeout> | null = null

  const connected = computed(() => connection.value?.connected === true)
  const connectionLoading = ref(false)
  const connectionChecked = ref(false)
  const storageUnavailable = ref(false)
  const connectionCheckFailed = ref(false)
  let connectionPromise: Promise<void> | null = null

  function readSelectedRepo(): SelectedRepository | null {
    try {
      const raw = localStorage.getItem('forgedesk.githubRepo')
      if (!raw) return null
      const parsed = JSON.parse(raw) as Partial<SelectedRepository>
      return parsed.repositoryId && parsed.owner && parsed.name && parsed.fullName
        ? (parsed as SelectedRepository)
        : null
    } catch {
      return null
    }
  }

  async function restoreSelectedRepository(): Promise<void> {
    try {
      const repository = await api.selectedRepository()
      selectedRepo.value = repository ? toSelected(repository) : null
    } catch {
      selectedRepo.value = null
    }
  }

  function toSelected(repository: Repository): SelectedRepository {
    return {
      repositoryId: repository.id,
      owner: repository.owner ?? repository.fullName.split('/')[0] ?? '',
      name: repository.name,
      fullName: repository.fullName,
    }
  }

  async function selectRepository(repository: Repository): Promise<boolean> {
    selectedRepo.value = toSelected(repository)
    try {
      localStorage.setItem('forgedesk.githubRepo', JSON.stringify(selectedRepo.value))
    } catch {
      // localStorage is a compatibility cache only.
    }
    try {
      await api.selectRepository(repository.id)
      return true
    } catch (caught) {
      selectedRepo.value = null
      const wire = toIpcError(caught)
      toast.error(t('githubRepos.selectError'), wire.message, wire.detail)
      return false
    }
  }

  function clearTimer(): void {
    if (timer) {
      clearTimeout(timer)
      timer = null
    }
  }

  async function loadConnection(force = false): Promise<void> {
    if (connectionPromise) {
      await connectionPromise
      return
    }
    if (connectionChecked.value && !force) return

    connectionLoading.value = true
    const task = (async () => {
      try {
        connection.value = await api.githubConnection()
        await restoreSelectedRepository()
        error.value = null
        storageUnavailable.value = false
        connectionCheckFailed.value = false
      } catch (caught) {
        const wire = toIpcError(caught)
        connection.value = null
        error.value = wire
        storageUnavailable.value = wire.code === 'credential_storage'
        connectionCheckFailed.value = !storageUnavailable.value
      } finally {
        connectionChecked.value = true
        connectionLoading.value = false
      }
    })()
    connectionPromise = task
    try {
      await task
    } finally {
      if (connectionPromise === task) connectionPromise = null
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
        connectionChecked.value = true
        connectionLoading.value = false
        error.value = null
        storageUnavailable.value = false
        connectionCheckFailed.value = false
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

  async function discoverRepositories(startPage = 1): Promise<void> {
    discovering.value = true
    try {
      const result = await api.githubDiscoverRepositories(startPage)
      repositories.value = startPage > 1 ? [...repositories.value, ...result.items] : result.items
      discoveryHasMore.value = result.hasMore
      discoveryPages.value = result.pagesLoaded
      discoveryNextPage.value = result.nextPage
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
    targetProjectId: string | null = null,
    startPage = 1,
  ): Promise<GitHubImportResult | null> {
    importing.value = true
    try {
      const result = await api.githubImportIssues(owner, repo, targetProjectId, startPage)
      let outboundSent = 0
      try {
        const outbound = await api.githubDispatchIssueOutbox(owner, repo)
        outboundSent = outbound.sent + outbound.reconciled
        if (outbound.outcomeUnknown > 0) {
          toast.warning(t('githubAuth.reconcileNeeded', { count: outbound.outcomeUnknown }))
        }
      } catch (caught) {
        const wire = toIpcError(caught)
        toast.warning(t('githubAuth.dispatchWarning', { message: wire.message }))
      }
      toast.success(
        t('githubRepos.imported', {
          imported: result.imported,
          skipped: result.skipped,
        }),
        t('githubRepos.importDetails', {
          project: result.projectName,
          updated: result.updated,
          conflicts: result.conflicted,
          sent: outboundSent,
        }),
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

  async function dispatchIssueOutbox(owner: string, repo: string): Promise<void> {
    syncingOutbox.value = true
    try {
      const result = await api.githubDispatchIssueOutbox(owner, repo)
      toast.success(
        t('settings.sync.outboxResult', {
          sent: result.sent,
          reconciled: result.reconciled,
          skipped: result.skipped,
        }),
        `${owner}/${repo}`,
      )
      if (result.outcomeUnknown > 0) {
        toast.warning(t('githubAuth.conflictReviewNeeded', { count: result.outcomeUnknown }))
      }
    } catch (caught) {
      const wire = toIpcError(caught)
      toast.error(t('settings.sync.outboxFailed'), wire.message, wire.detail)
    } finally {
      syncingOutbox.value = false
    }
  }

  async function disconnect(): Promise<void> {
    try {
      await api.githubDisconnect()
      connection.value = null
      connectionChecked.value = true
      connectionLoading.value = false
      error.value = null
      storageUnavailable.value = false
      connectionCheckFailed.value = false
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
    connectionLoading,
    connectionChecked,
    storageUnavailable,
    connectionCheckFailed,
    repositories,
    selectedRepo,
    discoveryHasMore,
    discoveryPages,
    discoveryNextPage,
    discovering,
    importing,
    syncingOutbox,
    loadConnection,
    start,
    poll,
    cancel,
    refresh,
    discoverRepositories,
    restoreSelectedRepository,
    importIssues,
    dispatchIssueOutbox,
    selectRepository,
    disconnect,
  }
})
