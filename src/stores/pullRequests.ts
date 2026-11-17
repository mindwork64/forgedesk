import { defineStore } from 'pinia'
import { ref } from 'vue'

import { i18n } from '@/i18n'
import { api, toIpcError } from '@/lib/ipc'
import { useToastStore } from '@/stores/toasts'
import type {
  GitHubPull,
  PullCapabilities,
  PullLink,
  PullRequestDetail,
} from '@/types/ipc'

export const usePullRequestStore = defineStore('pullRequests', () => {
  const t = i18n.global.t
  const toast = useToastStore()

  const pulls = ref<GitHubPull[]>([])
  const detail = ref<PullRequestDetail | null>(null)
  const linkedIssue = ref<PullLink | null>(null)
  const capabilities = ref<PullCapabilities | null>(null)
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
        try {
          const link = await api.githubPullLink(owner, repo, number)
          if (request === detailRequest) linkedIssue.value = link
        } catch {
          if (request === detailRequest) linkedIssue.value = null
        }
        try {
          const nextCapabilities = await api.githubPullCapabilities(owner, repo)
          if (request === detailRequest) capabilities.value = nextCapabilities
        } catch {
          if (request === detailRequest) capabilities.value = null
        }
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
    // Invalidate an in-flight detail request so a late response cannot revive
    // the pane after it has been closed or its repository has changed.
    detailRequest += 1
    selectedNumber.value = null
    detail.value = null
    linkedIssue.value = null
    capabilities.value = null
    detailLoading.value = false
  }

  async function linkIssue(
    owner: string,
    repo: string,
    number: number,
    issueKey: string,
  ): Promise<void> {
    linkedIssue.value = await api.githubLinkPull(owner, repo, number, issueKey)
    toast.success(t('prs.link.linked'), issueKey)
  }

  async function merge(
    owner: string,
    repo: string,
    number: number,
    method: string,
    expectedSha: string | null,
  ): Promise<boolean> {
    try {
      const result = await api.githubMergePull(
        owner,
        repo,
        number,
        method,
        expectedSha,
        true,
      )
      toast.success(t('prs.actions.merged'), result.message)
      await open(owner, repo, number)
      return true
    } catch (caught) {
      const wire = toIpcError(caught)
      toast.error(t('prs.actions.mergeError'), wire.message, wire.detail)
      return false
    }
  }

  async function closePull(owner: string, repo: string, number: number): Promise<boolean> {
    try {
      await api.githubClosePull(owner, repo, number, true)
      toast.success(t('prs.actions.closed'))
      await open(owner, repo, number)
      return true
    } catch (caught) {
      const wire = toIpcError(caught)
      toast.error(t('prs.actions.closeError'), wire.message, wire.detail)
      return false
    }
  }

  async function submitReview(
    owner: string,
    repo: string,
    number: number,
    event: string,
    body: string,
  ): Promise<boolean> {
    try {
      await api.githubSubmitReview(owner, repo, number, event, body, true)
      toast.success(t('prs.actions.reviewed'))
      await open(owner, repo, number)
      return true
    } catch (caught) {
      const wire = toIpcError(caught)
      toast.error(t('prs.actions.reviewError'), wire.message, wire.detail)
      return false
    }
  }

  async function unlinkIssue(owner: string, repo: string, number: number): Promise<void> {
    try {
      await api.githubUnlinkPull(owner, repo, number)
      linkedIssue.value = null
      toast.success(t('prs.link.unlinked'))
    } catch (caught) {
      toast.error(t('prs.link.unlinkError'), toIpcError(caught).message)
    }
  }

  return {
    pulls,
    detail,
    linkedIssue,
    capabilities,
    selectedNumber,
    loading,
    detailLoading,
    error,
    load,
    open,
    close,
    linkIssue,
    unlinkIssue,
    merge,
    closePull,
    submitReview,
  }
})
