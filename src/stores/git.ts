import { defineStore } from 'pinia'
import { ref } from 'vue'

import { i18n } from '@/i18n'
import { api, toIpcError } from '@/lib/ipc'
import { useToastStore } from '@/stores/toasts'
import type {
  GitAvailability,
  GitOverview,
  Repository,
  RepositoryInput,
} from '@/types/ipc'

export const useGitStore = defineStore('git', () => {
  const t = i18n.global.t
  const toast = useToastStore()

  const repositories = ref<Repository[]>([])
  const repository = ref<Repository | null>(null)
  const overview = ref<GitOverview | null>(null)
  const projectRepositories = ref<Repository[]>([])
  const projectRepositoriesLoading = ref(false)
  const availability = ref<GitAvailability | null>(null)
  const loading = ref(false)
  const error = ref<ReturnType<typeof toIpcError> | null>(null)
  let overviewRequest = 0

  async function loadAvailability(): Promise<void> {
    try {
      availability.value = await api.gitAvailable()
    } catch {
      availability.value = {
        available: false,
        version: null,
        error: 'Git availability could not be checked.',
      }
    }
  }

  async function loadRepositories(): Promise<void> {
    loading.value = true
    error.value = null
    try {
      repositories.value = await api.listRepositories()
    } catch (caught) {
      error.value = toIpcError(caught)
    } finally {
      loading.value = false
    }
  }

  async function loadProjectRepositories(projectId: string): Promise<void> {
    projectRepositoriesLoading.value = true
    try {
      projectRepositories.value = await api.getProjectRepositories(projectId)
    } catch (caught) {
      error.value = toIpcError(caught)
      projectRepositories.value = []
    } finally {
      projectRepositoriesLoading.value = false
    }
  }

  async function linkProjectRepository(
    repositoryId: string,
    projectId: string,
  ): Promise<boolean> {
    try {
      const updated = await api.linkRepository(repositoryId, projectId)
      projectRepositories.value = [
        ...projectRepositories.value.filter((item) => item.id !== updated.id),
        updated,
      ]
      repositories.value = repositories.value.map((item) =>
        item.id === updated.id ? updated : item,
      )
      return true
    } catch (caught) {
      const wire = toIpcError(caught)
      toast.error(t('repositories.linkError'), wire.message, wire.detail)
      return false
    }
  }

  async function unlinkProjectRepository(
    repositoryId: string,
    projectId: string,
  ): Promise<boolean> {
    try {
      const updated = await api.unlinkRepository(repositoryId, projectId)
      projectRepositories.value = projectRepositories.value.filter(
        (item) => item.id !== updated.id,
      )
      repositories.value = repositories.value.map((item) =>
        item.id === updated.id ? updated : item,
      )
      return true
    } catch (caught) {
      const wire = toIpcError(caught)
      toast.error(t('repositories.unlinkError'), wire.message, wire.detail)
      return false
    }
  }

  async function addRepository(input: RepositoryInput): Promise<Repository | null> {
    try {
      const repository = await api.addLocalRepository(input)
      repositories.value = [repository, ...repositories.value]
      toast.success(t('repositories.added'), repository.name)
      return repository
    } catch (caught) {
      const wire = toIpcError(caught)
      toast.error(t('repositories.addError'), wire.message, wire.detail)
      return null
    }
  }

  async function removeRepository(id: string): Promise<boolean> {
    try {
      await api.removeRepository(id)
      repositories.value = repositories.value.filter((item) => item.id !== id)
      if (overview.value?.repository.id === id) {
        overview.value = null
      }
      toast.success(t('repositories.removed'))
      return true
    } catch (caught) {
      const wire = toIpcError(caught)
      toast.error(t('repositories.removeError'), wire.message, wire.detail)
      return false
    }
  }

  async function loadRepository(id: string): Promise<void> {
    loading.value = true
    error.value = null
    overview.value = null
    try {
      repository.value = await api.getRepository(id)
    } catch (caught) {
      error.value = toIpcError(caught)
      repository.value = null
    } finally {
      loading.value = false
    }
  }

  async function loadOverview(id: string): Promise<void> {
    const request = ++overviewRequest
    loading.value = true
    error.value = null
    try {
      const result = await api.getGitOverview(id)
      if (request === overviewRequest) {
        overview.value = result
      }
    } catch (caught) {
      if (request === overviewRequest) {
        error.value = toIpcError(caught)
      }
    } finally {
      if (request === overviewRequest) {
        loading.value = false
      }
    }
  }

  async function createBranch(id: string, branch: string): Promise<boolean> {
    try {
      overview.value = await api.createGitBranch(id, branch)
      toast.success(t('repositories.branchCreated'), branch)
      return true
    } catch (caught) {
      const wire = toIpcError(caught)
      toast.error(t('repositories.branchError'), wire.message, wire.detail)
      return false
    }
  }

  async function switchBranch(id: string, branch: string): Promise<boolean> {
    try {
      overview.value = await api.switchGitBranch(id, branch)
      toast.success(t('repositories.branchSwitched'), branch)
      return true
    } catch (caught) {
      const wire = toIpcError(caught)
      toast.error(t('repositories.branchError'), wire.message, wire.detail)
      return false
    }
  }

  return {
    repositories,
    repository,
    overview,
    projectRepositories,
    projectRepositoriesLoading,
    availability,
    loading,
    error,
    loadAvailability,
    loadRepositories,
    loadRepository,
    loadProjectRepositories,
    linkProjectRepository,
    unlinkProjectRepository,
    addRepository,
    removeRepository,
    loadOverview,
    createBranch,
    switchBranch,
  }
})
