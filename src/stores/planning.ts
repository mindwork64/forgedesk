import { defineStore } from 'pinia'
import { ref } from 'vue'

import { i18n } from '@/i18n'
import { api } from '@/lib/ipc'
import { toIpcError } from '@/lib/ipc'
import { useToastStore } from '@/stores/toasts'
import type {
  Cycle,
  CycleDetail,
  CycleInput,
  Document,
  DocumentInput,
  Initiative,
  InitiativeDetail,
  InitiativeInput,
  MilestoneInput,
  Project,
  ProjectDetail,
  ProjectInput,
  ProjectQuery,
} from '@/types/ipc'

export const usePlanningStore = defineStore('planning', () => {
  const t = i18n.global.t
  const toast = useToastStore()

  const projects = ref<Project[]>([])
  const initiatives = ref<Initiative[]>([])
  const cycles = ref<Cycle[]>([])
  const documents = ref<Document[]>([])

  const project = ref<ProjectDetail | null>(null)
  const initiative = ref<InitiativeDetail | null>(null)
  const cycle = ref<CycleDetail | null>(null)
  const document = ref<Document | null>(null)

  const loading = ref(false)
  const error = ref<ReturnType<typeof toIpcError> | null>(null)

  let projectRequest = 0
  let initiativeRequest = 0
  let cycleRequest = 0
  let documentRequest = 0

  async function loadProjects(query: ProjectQuery = {}): Promise<void> {
    loading.value = true
    error.value = null
    try {
      projects.value = await api.listProjectCards(query)
    } catch (caught) {
      error.value = toIpcError(caught)
    } finally {
      loading.value = false
    }
  }

  async function loadProject(id: string): Promise<void> {
    const request = ++projectRequest
    loading.value = true
    error.value = null
    try {
      const result = await api.getProject(id)
      if (request === projectRequest) {
        project.value = result
      }
    } catch (caught) {
      if (request === projectRequest) {
        error.value = toIpcError(caught)
      }
    } finally {
      if (request === projectRequest) {
        loading.value = false
      }
    }
  }

  async function createProject(input: ProjectInput): Promise<Project | null> {
    try {
      const created = await api.createProject(input)
      projects.value = [created, ...projects.value]
      toast.success(t('projects.created'), created.name)
      return created
    } catch (caught) {
      const wire = toIpcError(caught)
      toast.error(t('projects.createError'), wire.message, wire.detail)
      return null
    }
  }

  async function updateProject(
    id: string,
    input: ProjectInput,
  ): Promise<Project | null> {
    try {
      const updated = await api.updateProject(id, input)
      projects.value = projects.value.map((item) =>
        item.id === id ? updated : item,
      )
      return updated
    } catch (caught) {
      const wire = toIpcError(caught)
      toast.error(t('projects.updateError'), wire.message, wire.detail)
      return null
    }
  }

  async function archiveProject(id: string, archived = true): Promise<boolean> {
    try {
      await api.archiveProject(id, archived)
      projects.value = projects.value.filter((item) => item.id !== id)
      if (project.value?.project.id === id) {
        project.value = null
      }
      toast.success(archived ? t('projects.archived') : t('projects.restored'))
      return true
    } catch (caught) {
      const wire = toIpcError(caught)
      toast.error(t('projects.archiveError'), wire.message, wire.detail)
      return false
    }
  }

  async function createMilestone(
    projectId: string,
    input: MilestoneInput,
  ): Promise<boolean> {
    try {
      await api.createMilestone(projectId, input)
      await loadProject(projectId)
      toast.success(t('projects.milestoneCreated'))
      return true
    } catch (caught) {
      const wire = toIpcError(caught)
      toast.error(t('projects.milestoneError'), wire.message, wire.detail)
      return false
    }
  }

  async function updateMilestone(
    projectId: string,
    milestoneId: string,
    input: MilestoneInput,
  ): Promise<boolean> {
    try {
      await api.updateMilestone(milestoneId, input)
      await loadProject(projectId)
      return true
    } catch (caught) {
      const wire = toIpcError(caught)
      toast.error(t('projects.milestoneError'), wire.message, wire.detail)
      return false
    }
  }

  async function assignMilestoneIssue(
    projectId: string,
    milestoneId: string,
    issueId: string,
    assigned: boolean,
  ): Promise<boolean> {
    try {
      await api.assignMilestoneIssue(milestoneId, issueId, assigned)
      await loadProject(projectId)
      return true
    } catch (caught) {
      const wire = toIpcError(caught)
      toast.error(t('projects.milestoneError'), wire.message, wire.detail)
      return false
    }
  }

  async function loadInitiatives(): Promise<void> {
    loading.value = true
    error.value = null
    try {
      initiatives.value = await api.listInitiatives()
    } catch (caught) {
      error.value = toIpcError(caught)
    } finally {
      loading.value = false
    }
  }

  async function loadInitiative(id: string): Promise<void> {
    const request = ++initiativeRequest
    loading.value = true
    error.value = null
    try {
      const result = await api.getInitiative(id)
      if (request === initiativeRequest) {
        initiative.value = result
      }
    } catch (caught) {
      if (request === initiativeRequest) {
        error.value = toIpcError(caught)
      }
    } finally {
      if (request === initiativeRequest) {
        loading.value = false
      }
    }
  }

  async function createInitiative(
    input: InitiativeInput,
  ): Promise<Initiative | null> {
    try {
      const created = await api.createInitiative(input)
      initiatives.value = [created, ...initiatives.value]
      toast.success(t('initiatives.created'), created.name)
      return created
    } catch (caught) {
      const wire = toIpcError(caught)
      toast.error(t('initiatives.createError'), wire.message, wire.detail)
      return null
    }
  }

  async function updateInitiative(
    id: string,
    input: InitiativeInput,
  ): Promise<Initiative | null> {
    try {
      const updated = await api.updateInitiative(id, input)
      initiatives.value = initiatives.value.map((item) =>
        item.id === id ? updated : item,
      )
      return updated
    } catch (caught) {
      const wire = toIpcError(caught)
      toast.error(t('initiatives.updateError'), wire.message, wire.detail)
      return null
    }
  }

  async function loadCycles(): Promise<void> {
    loading.value = true
    error.value = null
    try {
      cycles.value = await api.listCycles()
    } catch (caught) {
      error.value = toIpcError(caught)
    } finally {
      loading.value = false
    }
  }

  async function loadCycle(id: string): Promise<void> {
    const request = ++cycleRequest
    loading.value = true
    error.value = null
    try {
      const result = await api.getCycle(id)
      if (request === cycleRequest) {
        cycle.value = result
      }
    } catch (caught) {
      if (request === cycleRequest) {
        error.value = toIpcError(caught)
      }
    } finally {
      if (request === cycleRequest) {
        loading.value = false
      }
    }
  }

  async function createCycle(input: CycleInput): Promise<Cycle | null> {
    try {
      const created = await api.createCycle(input)
      cycles.value = [created, ...cycles.value]
      toast.success(t('cycles.created'), created.name)
      return created
    } catch (caught) {
      const wire = toIpcError(caught)
      toast.error(t('cycles.createError'), wire.message, wire.detail)
      return null
    }
  }

  async function updateCycle(
    id: string,
    input: CycleInput,
  ): Promise<Cycle | null> {
    try {
      const updated = await api.updateCycle(id, input)
      cycles.value = cycles.value.map((item) =>
        item.id === id ? updated : item,
      )
      return updated
    } catch (caught) {
      const wire = toIpcError(caught)
      toast.error(t('cycles.updateError'), wire.message, wire.detail)
      return null
    }
  }

  async function assignCycleIssue(
    cycleId: string,
    issueId: string,
    assigned: boolean,
  ): Promise<boolean> {
    try {
      await api.assignCycleIssue(cycleId, issueId, assigned)
      await loadCycle(cycleId)
      return true
    } catch (caught) {
      const wire = toIpcError(caught)
      toast.error(t('cycles.updateError'), wire.message, wire.detail)
      return false
    }
  }

  async function loadDocuments(): Promise<void> {
    loading.value = true
    error.value = null
    try {
      documents.value = await api.listDocuments()
    } catch (caught) {
      error.value = toIpcError(caught)
    } finally {
      loading.value = false
    }
  }

  async function selectDocument(id: string): Promise<void> {
    const request = ++documentRequest
    try {
      const result = await api.getDocument(id)
      if (request === documentRequest) {
        document.value = result
      }
    } catch (caught) {
      toast.error(t('documents.loadError'), toIpcError(caught).message)
    }
  }

  async function createDocument(input: DocumentInput): Promise<Document | null> {
    try {
      const created = await api.createDocument(input)
      documents.value = [created, ...documents.value]
      document.value = created
      toast.success(t('documents.created'), created.title)
      return created
    } catch (caught) {
      const wire = toIpcError(caught)
      toast.error(t('documents.createError'), wire.message, wire.detail)
      return null
    }
  }

  async function updateDocument(
    id: string,
    input: DocumentInput,
  ): Promise<Document | null> {
    try {
      const updated = await api.updateDocument(id, input)
      documents.value = documents.value.map((item) =>
        item.id === id ? updated : item,
      )
      document.value = updated
      return updated
    } catch (caught) {
      const wire = toIpcError(caught)
      toast.error(t('documents.updateError'), wire.message, wire.detail)
      return null
    }
  }

  async function deleteDocument(id: string): Promise<boolean> {
    try {
      await api.deleteDocument(id)
      documents.value = documents.value.filter((item) => item.id !== id)
      if (document.value?.id === id) {
        document.value = null
      }
      toast.success(t('documents.deleted'))
      return true
    } catch (caught) {
      const wire = toIpcError(caught)
      toast.error(t('documents.deleteError'), wire.message, wire.detail)
      return false
    }
  }

  async function exportDocument(id: string): Promise<string | null> {
    try {
      return await api.exportDocument(id)
    } catch (caught) {
      const wire = toIpcError(caught)
      toast.error(t('documents.exportError'), wire.message, wire.detail)
      return null
    }
  }

  return {
    projects,
    initiatives,
    cycles,
    documents,
    project,
    initiative,
    cycle,
    document,
    loading,
    error,
    loadProjects,
    loadProject,
    createProject,
    updateProject,
    archiveProject,
    createMilestone,
    updateMilestone,
    assignMilestoneIssue,
    loadInitiatives,
    loadInitiative,
    createInitiative,
    updateInitiative,
    loadCycles,
    loadCycle,
    createCycle,
    updateCycle,
    assignCycleIssue,
    loadDocuments,
    selectDocument,
    createDocument,
    updateDocument,
    deleteDocument,
    exportDocument,
  }
})
