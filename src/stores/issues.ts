import { defineStore } from 'pinia'
import { computed, ref } from 'vue'

import { i18n } from '@/i18n'
import { api, toIpcError } from '@/lib/ipc'
import type {
  BatchIssueInput,
  CreateIssueInput,
  IssueDetail,
  IssueQuery,
  IssueSummary,
  IpcError,
  Label,
  ProjectRef,
  RelationInputKind,
  SavedView,
  SavedViewInput,
  SortSpec,
  StatusSummary,
  UpdateIssueInput,
  Workflow,
} from '@/types/ipc'

import { useToastStore } from './toasts'

const t = (key: string, params?: Record<string, unknown>): string => {
  const translate = i18n.global.t as (
    key: string,
    params?: Record<string, unknown>,
  ) => string
  return params ? translate(key, params) : translate(key)
}

const VIEW_KEY = 'fd.issues.view'

function loadView(): 'list' | 'board' {
  try {
    return localStorage.getItem(VIEW_KEY) === 'board' ? 'board' : 'list'
  } catch {
    return 'list'
  }
}

export const useIssueStore = defineStore('issues', () => {
  const workflow = ref<Workflow | null>(null)
  const labels = ref<Label[]>([])
  const projects = ref<ProjectRef[]>([])
  const issues = ref<IssueSummary[]>([])
  const loading = ref(false)
  const error = ref<IpcError | null>(null)
  const view = ref<'list' | 'board'>(loadView())
  const search = ref('')
  const filter = ref<IssueQuery['filter']>(null)
  const sort = ref<SortSpec[]>([
    { field: 'updatedAt', direction: 'desc' },
  ])
  const groupBy = ref<string | null>(null)
  const savedViews = ref<SavedView[]>([])
  const activeViewId = ref<string | null>(null)
  const selectedId = ref<string | null>(null)
  const detail = ref<IssueDetail | null>(null)
  const detailLoading = ref(false)

  const toast = useToastStore()

  let searchTimer: ReturnType<typeof setTimeout> | null = null
  let listRequestId = 0
  let detailRequestId = 0
  let mutationQueue: Promise<unknown> = Promise.resolve()

  function enqueue<T>(operation: () => Promise<T>): Promise<T> {
    const run = mutationQueue.then(operation, operation)
    mutationQueue = run.then(
      () => undefined,
      () => undefined,
    )
    return run
  }

  const statuses = computed<StatusSummary[]>(
    () => workflow.value?.statuses ?? [],
  )
  const isEmpty = computed(
    () => !loading.value && error.value === null && issues.value.length === 0,
  )
  const selectedSummary = computed<IssueSummary | null>(() => {
    if (detail.value) {
      return detail.value.summary
    }
    return issues.value.find((issue) => issue.id === selectedId.value) ?? null
  })
  const board = computed(() =>
    statuses.value.map((status) => ({
      status,
      issues: issues.value.filter((issue) => issue.status.id === status.id),
    })),
  )
  const pinnedViews = computed(() =>
    savedViews.value.filter((view) => view.isPinned),
  )
  const activeView = computed(
    () =>
      savedViews.value.find((view) => view.id === activeViewId.value) ?? null,
  )

  async function loadReference(): Promise<void> {
    const [nextWorkflow, nextLabels, nextProjects] = await Promise.all([
      api.listWorkflow(),
      api.listLabels(),
      api.listProjects(),
    ])
    workflow.value = nextWorkflow
    labels.value = nextLabels
    projects.value = nextProjects
  }

  async function load(): Promise<void> {
    const requestId = ++listRequestId
    loading.value = true
    error.value = null
    try {
      if (!workflow.value) {
        await loadReference()
      }
      const query: IssueQuery = {
        filter: filter.value,
        sort: sort.value.length
          ? sort.value
          : [{ field: 'updatedAt', direction: 'desc' }],
        search: search.value.trim() || null,
        includeArchived: false,
        limit: 500,
      }
      const list = await api.listIssues(query)
      if (requestId === listRequestId) {
        issues.value = list
      }
    } catch (caught) {
      if (requestId === listRequestId) {
        error.value = toIpcError(caught)
      }
    } finally {
      if (requestId === listRequestId) {
        loading.value = false
      }
    }
  }

  function setFilter(next: IssueQuery['filter']): void {
    filter.value = next
    activeViewId.value = null
    void load()
  }

  function setSort(next: SortSpec[]): void {
    sort.value = next.length
      ? next
      : [{ field: 'updatedAt', direction: 'desc' }]
    activeViewId.value = null
    void load()
  }

  function setGroupBy(next: string | null): void {
    groupBy.value = next
    activeViewId.value = null
  }

  async function loadSavedViews(): Promise<void> {
    try {
      savedViews.value = await api.listSavedViews()
    } catch (caught) {
      const wire = toIpcError(caught)
      // A missing workspace simply means there is nothing to pin yet.
      if (wire.code !== 'not_found') {
        toast.error(t('views.loadError'), wire.message)
      }
    }
  }

  function applyView(view: SavedView): void {
    filter.value = view.filters
    sort.value = view.sort.length
      ? view.sort
      : [{ field: 'updatedAt', direction: 'desc' }]
    groupBy.value = view.groupBy
    activeViewId.value = view.id
    void load()
  }

  async function saveView(
    name: string,
    overwriteId: string | null = null,
  ): Promise<SavedView | null> {
    const existing = overwriteId
      ? savedViews.value.find((view) => view.id === overwriteId) ?? null
      : null
    const input: SavedViewInput = {
      name,
      filters: filter.value,
      sort: sort.value,
      groupBy: groupBy.value,
      columns: [
        'key',
        'title',
        'status',
        'priority',
        'project',
        'labels',
        'updated',
      ],
      density: 'standard',
      isPinned: existing?.isPinned ?? false,
    }
    try {
      const saved = overwriteId
        ? await api.updateSavedView(overwriteId, input)
        : await api.createSavedView(input)
      await loadSavedViews()
      activeViewId.value = saved.id
      toast.success(t('views.saved'), saved.name)
      return saved
    } catch (caught) {
      const wire = toIpcError(caught)
      toast.error(t('views.saveError'), wire.message, wire.detail)
      return null
    }
  }

  async function deleteView(id: string): Promise<boolean> {
    try {
      await api.deleteSavedView(id)
      if (activeViewId.value === id) {
        activeViewId.value = null
      }
      await loadSavedViews()
      toast.success(t('views.deleted'))
      return true
    } catch (caught) {
      const wire = toIpcError(caught)
      toast.error(t('views.deleteError'), wire.message, wire.detail)
      return false
    }
  }

  async function togglePin(view: SavedView): Promise<boolean> {
    const input: SavedViewInput = {
      name: view.name,
      filters: view.filters,
      sort: view.sort,
      groupBy: view.groupBy,
      columns: view.columns,
      density: view.density,
      isPinned: !view.isPinned,
    }
    try {
      await api.updateSavedView(view.id, input)
      await loadSavedViews()
      return true
    } catch (caught) {
      const wire = toIpcError(caught)
      toast.error(t('views.pinError'), wire.message, wire.detail)
      return false
    }
  }

  function setView(next: 'list' | 'board'): void {
    view.value = next
    try {
      localStorage.setItem(VIEW_KEY, next)
    } catch {
      // View preference is a convenience; ignore storage failures.
    }
  }

  function setSearch(value: string): void {
    search.value = value
    if (searchTimer) {
      clearTimeout(searchTimer)
    }
    searchTimer = setTimeout(() => {
      searchTimer = null
      void load()
    }, 200)
  }

  function replaceSummary(summary: IssueSummary): void {
    const index = issues.value.findIndex((issue) => issue.id === summary.id)
    if (index >= 0) {
      issues.value[index] = summary
    } else {
      issues.value = [summary, ...issues.value]
    }
  }

  async function selectIssue(id: string): Promise<void> {
    const requestId = ++detailRequestId
    selectedId.value = id
    detail.value = null
    detailLoading.value = true
    try {
      const loaded = await api.getIssue(id)
      if (requestId === detailRequestId && selectedId.value === id) {
        detail.value = loaded
      }
    } catch (caught) {
      if (requestId === detailRequestId) {
        toast.error(t('issues.detailError'), toIpcError(caught).message)
      }
    } finally {
      if (requestId === detailRequestId) {
        detailLoading.value = false
      }
    }
  }

  function closeDetail(): void {
    selectedId.value = null
    detail.value = null
  }

  async function createIssue(input: CreateIssueInput): Promise<IssueDetail> {
    const created = await enqueue(() => api.createIssue(input))
    replaceSummary(created.summary)
    detail.value = created
    selectedId.value = created.summary.id
    void loadReference()
    toast.success(t('issues.created'), created.summary.key)
    return created
  }

  async function updateIssue(
    id: string,
    input: UpdateIssueInput,
  ): Promise<boolean> {
    try {
      const updated = await enqueue(() => api.updateIssue(id, input))
      if (selectedId.value === id) {
        detail.value = updated
      }
      replaceSummary(updated.summary)
      labels.value = await api.listLabels()
      return true
    } catch (caught) {
      const wire = toIpcError(caught)
      toast.error(t('issues.updateError'), wire.message, wire.detail)
      return false
    }
  }

  async function archiveIssue(
    id: string,
    archived = true,
  ): Promise<boolean> {
    try {
      const updated = await enqueue(() => api.archiveIssue(id, archived))
      if (selectedId.value === id) {
        detail.value = updated
      }
      if (archived) {
        issues.value = issues.value.filter((issue) => issue.id !== id)
        if (selectedId.value === id) {
          selectedId.value = null
          detail.value = null
        }
      } else {
        replaceSummary(updated.summary)
      }
      toast.success(
        archived ? t('issues.archived') : t('issues.unarchived'),
        updated.summary.key,
      )
      return true
    } catch (caught) {
      toast.error(t('issues.archiveError'), toIpcError(caught).message)
      return false
    }
  }

  async function duplicateIssue(id: string): Promise<IssueDetail | null> {
    try {
      const copy = await enqueue(() => api.duplicateIssue(id))
      replaceSummary(copy.summary)
      detail.value = copy
      selectedId.value = copy.summary.id
      toast.success(t('issues.duplicated'), copy.summary.key)
      return copy
    } catch (caught) {
      toast.error(t('issues.duplicateError'), toIpcError(caught).message)
      return null
    }
  }

  async function addComment(id: string, body: string): Promise<boolean> {
    try {
      const updated = await enqueue(() => api.addIssueComment(id, body))
      detail.value = updated
      replaceSummary(updated.summary)
      return true
    } catch (caught) {
      toast.error(t('issues.commentError'), toIpcError(caught).message)
      return false
    }
  }

  async function addRelation(
    id: string,
    relatedIssueId: string,
    kind: RelationInputKind,
  ): Promise<boolean> {
    try {
      const updated = await enqueue(() => api.addIssueRelation(id, relatedIssueId, kind))
      detail.value = updated
      return true
    } catch (caught) {
      toast.error(t('issues.relationError'), toIpcError(caught).message)
      return false
    }
  }

  async function removeRelation(
    id: string,
    relationId: string,
  ): Promise<boolean> {
    try {
      const updated = await enqueue(() => api.removeIssueRelation(id, relationId))
      detail.value = updated
      return true
    } catch (caught) {
      toast.error(t('issues.relationError'), toIpcError(caught).message)
      return false
    }
  }

  async function batchUpdate(
    ids: string[],
    input: BatchIssueInput,
  ): Promise<boolean> {
    try {
      const updated = await enqueue(() => api.batchUpdateIssues(ids, input))
      const byId = new Map(
        updated.map((detail) => [detail.summary.id, detail.summary]),
      )
      const archivedIds = new Set(
        updated
          .filter((detail) => detail.summary.archivedAt)
          .map((detail) => detail.summary.id),
      )
      issues.value = issues.value
        .filter((issue) => !archivedIds.has(issue.id))
        .map((issue) => byId.get(issue.id) ?? issue)
      if (selectedId.value && archivedIds.has(selectedId.value)) {
        closeDetail()
      }
      return true
    } catch (caught) {
      toast.error(t('issues.updateError'), toIpcError(caught).message)
      return false
    }
  }

  return {
    workflow,
    labels,
    projects,
    issues,
    loading,
    error,
    view,
    search,
    filter,
    sort,
    groupBy,
    savedViews,
    activeViewId,
    selectedId,
    detail,
    detailLoading,
    statuses,
    isEmpty,
    selectedSummary,
    board,
    pinnedViews,
    activeView,
    load,
    loadReference,
    loadSavedViews,
    applyView,
    saveView,
    deleteView,
    togglePin,
    setFilter,
    setSort,
    setGroupBy,
    setView,
    setSearch,
    selectIssue,
    closeDetail,
    createIssue,
    updateIssue,
    archiveIssue,
    duplicateIssue,
    addComment,
    addRelation,
    removeRelation,
    batchUpdate,
  }
})
