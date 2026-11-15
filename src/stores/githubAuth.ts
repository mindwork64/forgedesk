import { defineStore } from 'pinia'
import { computed, ref } from 'vue'

import { i18n } from '@/i18n'
import { api, toIpcError } from '@/lib/ipc'
import { useToastStore } from '@/stores/toasts'
import type { DeviceFlowStart, GitHubConnection } from '@/types/ipc'

export const useGithubAuthStore = defineStore('githubAuth', () => {
  const t = i18n.global.t
  const toast = useToastStore()

  const connection = ref<GitHubConnection | null>(null)
  const flow = ref<DeviceFlowStart | null>(null)
  const polling = ref(false)
  const starting = ref(false)
  const error = ref<ReturnType<typeof toIpcError> | null>(null)
  let timer: ReturnType<typeof setTimeout> | null = null

  const connected = computed(() => connection.value?.connected === true)

  function clearTimer(): void {
    if (timer) {
      clearTimeout(timer)
      timer = null
    }
  }

  async function loadConnection(): Promise<void> {
    try {
      connection.value = await api.githubConnection()
    } catch (caught) {
      error.value = toIpcError(caught)
    }
  }

  async function start(): Promise<void> {
    starting.value = true
    error.value = null
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

  async function disconnect(): Promise<void> {
    try {
      await api.githubDisconnect()
      connection.value = null
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
    loadConnection,
    start,
    poll,
    cancel,
    refresh,
    disconnect,
  }
})
