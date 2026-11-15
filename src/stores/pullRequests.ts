import { defineStore } from 'pinia'
import { ref } from 'vue'

import { i18n } from '@/i18n'
import { api, toIpcError } from '@/lib/ipc'
import { useToastStore } from '@/stores/toasts'
import type { GitHubPull, PullRequestDetail } from '@/types/ipc'

export const usePullRequestStore = defineStore('pullRequests', () => {
  const t = i18n.global.t
  const toast = useToastStore()

  const pulls = ref<GitHubPull[]>([])
  const detail = ref<PullRequestDetail | null>(null)
  const selectedNumber = ref<number | null>(null)
  const loading = ref(false)
  const detailLoading = ref(false)
  const error = ref<ReturnType<typeof toIpcError> | null>(null)
  let listRequest = 0
  let detailRequest = 0

  async function load(
    owner: string,
    repo: string,
    pullState: 'open' | 'closed' | 'all',
  ): Promise<void> {
    const request = ++listRequest
    loading.value = true
    error.value = null
    try {
      const result = await api.githubListPulls(owner, repo, pullState)
      if (request === listRequest) {
        pulls.value = result
      }
    } catch (caught) {
      if (request === listRequest) {
        error.value = toIpcError(caught)
      }
    } finally {
      if (request === listRequest) {
        loading.value = false
      }
    }
  }

  async function open(owner: string, repo: string, number: number): Promise<void> {
    const request = ++detailRequest
    selectedNumber.value = number
    detail.value = null
    detailLoading.value = true
    try {
      const result = await api.githubPullDetail(owner, repo, number)
      if (request === detailRequest) {
        detail.value = result
      }
    } catch (caught) {
      if (request === detailRequest) {
        toast.error(t('prs.loadError'), toIpcError(caught).message)
      }
    } finally {
      if (request === detailRequest) {
        detailLoading.value = false
      }
    }
  }

  function close(): void {
    selectedNumber.value = null
    detail.value = null
  }

  return {
    pulls,
    detail,
    selectedNumber,
    loading,
    detailLoading,
    error,
    load,
    open,
    close,
  }
})
