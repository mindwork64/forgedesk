<script setup lang="ts">
import { CircleDot, FileText, FolderGit2, Layers } from '@lucide/vue'
import type { Component } from 'vue'
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRouter } from 'vue-router'

import PageHeader from '@/components/layout/PageHeader.vue'
import PageToolbar from '@/components/layout/PageToolbar.vue'
import FdEmptyState from '@/components/ui/FdEmptyState.vue'
import FdErrorState from '@/components/ui/FdErrorState.vue'
import FdInput from '@/components/ui/FdInput.vue'
import FdLoadingSkeleton from '@/components/ui/FdLoadingSkeleton.vue'
import { api, toIpcError } from '@/lib/ipc'
import type { Document, IssueSummary, Project, Repository } from '@/types/ipc'

type ResultKind = 'issue' | 'project' | 'document' | 'repository'
interface SearchResult {
  id: string
  kind: ResultKind
  title: string
  subtitle: string
  description: string
}
interface SearchGroup {
  kind: ResultKind
  label: string
  icon: Component
  results: SearchResult[]
}

const router = useRouter()
const { t } = useI18n()
const query = ref('')
const documents = ref<Document[]>([])
const repositories = ref<Repository[]>([])
const projects = ref<Project[]>([])
const issues = ref<IssueSummary[]>([])
const catalogsLoading = ref(true)
const searchLoading = ref(false)
const catalogsError = ref<ReturnType<typeof toIpcError> | null>(null)
const searchError = ref<ReturnType<typeof toIpcError> | null>(null)
let searchTimer: ReturnType<typeof setTimeout> | null = null
let requestId = 0

const needle = computed(() => query.value.trim().toLocaleLowerCase())
const hasQuery = computed(() => query.value.trim().length >= 2)

const groups = computed((): SearchGroup[] => {
  const q = needle.value
  if (q.length < 2) return []

  const issueResults: SearchResult[] = issues.value.slice(0, 12).map((issue) => ({
    id: issue.id,
    kind: 'issue',
    title: issue.title,
    subtitle: issue.key + ' · ' + issue.status.name,
    description: issue.project?.name ?? '',
  }))
  const projectResults: SearchResult[] = projects.value.slice(0, 12).map((project) => ({
    id: project.id,
    kind: 'project',
    title: project.name,
    subtitle: t('searchPage.projectStatus', { status: t(`planning.status.${project.status}`) }),
    description: project.description,
  }))
  const documentResults: SearchResult[] = documents.value
    .filter((document) => !document.archivedAt)
    .filter((document) => (document.title + ' ' + document.body).toLocaleLowerCase().includes(q))
    .slice(0, 12)
    .map((document) => ({
      id: document.id,
      kind: 'document',
      title: document.title,
      subtitle: [document.projectName, document.initiativeName].filter(Boolean).join(' · '),
      description: document.body.replace(/\s+/g, ' ').slice(0, 180),
    }))
  const repositoryResults: SearchResult[] = repositories.value
    .filter((repository) => !repository.isArchived)
    .filter((repository) =>
      [
        repository.fullName,
        repository.name,
        repository.owner ?? '',
        repository.description ?? '',
        repository.cloneUrl ?? '',
        repository.htmlUrl ?? '',
        ...repository.checkouts.map((checkout) => checkout.path),
      ].join(' ').toLocaleLowerCase().includes(q),
    )
    .slice(0, 12)
    .map((repository) => ({
      id: repository.id,
      kind: 'repository',
      title: repository.fullName,
      subtitle: t(`searchPage.providers.${repository.provider}`),
      description: repository.description ?? repository.checkouts[0]?.path ?? '',
    }))

  return [
    { kind: 'issue', label: t('searchPage.issues'), icon: CircleDot, results: issueResults },
    { kind: 'project', label: t('searchPage.projects'), icon: Layers, results: projectResults },
    { kind: 'document', label: t('searchPage.documents'), icon: FileText, results: documentResults },
    { kind: 'repository', label: t('searchPage.repositories'), icon: FolderGit2, results: repositoryResults },
  ].filter((group) => group.results.length > 0)
})

const resultCount = computed(() => groups.value.reduce((total, group) => total + group.results.length, 0))

async function loadCatalogs(): Promise<void> {
  catalogsLoading.value = true
  catalogsError.value = null
  const [documentResult, repositoryResult] = await Promise.allSettled([
    api.listDocuments(),
    api.listRepositories(),
  ])
  if (documentResult.status === 'fulfilled') documents.value = documentResult.value
  else catalogsError.value = toIpcError(documentResult.reason)
  if (repositoryResult.status === 'fulfilled') repositories.value = repositoryResult.value
  else if (!catalogsError.value) catalogsError.value = toIpcError(repositoryResult.reason)
  catalogsLoading.value = false
}

async function runSearch(raw: string): Promise<void> {
  const term = raw.trim()
  const currentRequest = ++requestId
  if (term.length < 2) {
    issues.value = []
    projects.value = []
    searchError.value = null
    searchLoading.value = false
    return
  }

  searchLoading.value = true
  searchError.value = null
  const [issueResult, projectResult] = await Promise.allSettled([
    api.listIssues({ search: term, includeArchived: false, limit: 40 }),
    api.listProjectCards({ search: term, includeArchived: false }),
  ])
  if (currentRequest !== requestId) return

  if (issueResult.status === 'fulfilled') issues.value = issueResult.value
  else {
    issues.value = []
    searchError.value = toIpcError(issueResult.reason)
  }
  if (projectResult.status === 'fulfilled') projects.value = projectResult.value
  else if (!searchError.value) searchError.value = toIpcError(projectResult.reason)
  searchLoading.value = false
}

watch(query, (value) => {
  searchLoading.value = value.trim().length >= 2
  if (searchTimer) clearTimeout(searchTimer)
  searchTimer = setTimeout(() => {
    searchTimer = null
    void runSearch(value)
  }, 180)
})

function openResult(result: SearchResult): void {
  switch (result.kind) {
    case 'issue':
      void router.push({ path: '/my-issues', query: { issue: result.id } })
      break
    case 'project':
      void router.push(`/projects/${result.id}`)
      break
    case 'document':
      void router.push({ path: '/documents', query: { doc: result.id } })
      break
    case 'repository':
      void router.push(`/repositories/${result.id}`)
      break
  }
}

function retry(): void {
  void loadCatalogs()
  void runSearch(query.value)
}

onMounted(() => {
  void loadCatalogs()
})

onBeforeUnmount(() => {
  if (searchTimer) clearTimeout(searchTimer)
  requestId += 1
})
</script>

<template>
  <div class="fd-search-page">
    <PageHeader :title="t('pages.search.title')" :description="t('pages.search.description')" />
    <PageToolbar>
      <template #left>
        <FdInput
          v-model="query"
          type="search"
          class="fd-search-input"
          autofocus
          :placeholder="t('searchPage.placeholder')"
          :aria-label="t('searchPage.placeholder')"
        />
      </template>
      <template #right>
        <span v-if="hasQuery && !searchLoading" class="fd-search-count">
          {{ t('searchPage.resultCount', { count: resultCount }) }}
        </span>
      </template>
    </PageToolbar>

    <div class="fd-search-scroll">
      <div v-if="catalogsLoading" class="fd-search-state">
        <FdLoadingSkeleton :rows="5" :height="48" />
      </div>
      <div v-else-if="catalogsError && !hasQuery" class="fd-search-state">
        <FdErrorState
          :title="t('searchPage.loadError')"
          :message="catalogsError.message"
          :code="catalogsError.code"
          :detail="catalogsError.detail"
          @retry="retry"
        />
      </div>
      <div v-else-if="!hasQuery" class="fd-search-state">
        <FdEmptyState :icon="CircleDot" :title="t('searchPage.startTitle')" :body="t('searchPage.startBody')" />
      </div>
      <div v-else-if="searchLoading" class="fd-search-state">
        <FdLoadingSkeleton :rows="6" :height="48" />
      </div>
      <div v-else-if="searchError" class="fd-search-state">
        <FdErrorState
          :title="t('searchPage.searchError')"
          :message="searchError.message"
          :code="searchError.code"
          :detail="searchError.detail"
          @retry="retry"
        />
      </div>
      <div v-else-if="resultCount === 0" class="fd-search-state">
        <FdEmptyState :icon="CircleDot" :title="t('searchPage.emptyTitle')" :body="t('searchPage.emptyBody')" />
      </div>
      <div v-else class="fd-search-results" data-testid="search-results">
        <section v-for="group in groups" :key="group.kind" class="fd-search-group">
          <h2>
            <component :is="group.icon" :size="14" aria-hidden="true" />
            {{ group.label }}
            <span>{{ group.results.length }}</span>
          </h2>
          <button
            v-for="result in group.results"
            :key="`${result.kind}:${result.id}`"
            type="button"
            class="fd-search-result"
            :data-testid="`search-result-${result.kind}`"
            @click="openResult(result)"
          >
            <span class="fd-search-result__main">
              <strong>{{ result.title }}</strong>
              <span v-if="result.subtitle">{{ result.subtitle }}</span>
              <small v-if="result.description">{{ result.description }}</small>
            </span>
            <span class="fd-search-result__open">{{ t('searchPage.open') }}</span>
          </button>
        </section>
      </div>
    </div>
  </div>
</template>

<style scoped>
.fd-search-page { display: flex; flex-direction: column; width: 100%; height: 100%; min-width: 0; min-height: 0; overflow: hidden; }
.fd-search-input { width: min(560px, 55vw); }
.fd-search-count { color: var(--fd-text-muted); font-size: var(--fd-text-11); white-space: nowrap; }
.fd-search-scroll { flex: 1; min-height: 0; overflow: auto; }
.fd-search-state { display: flex; justify-content: center; padding: var(--fd-space-6) var(--fd-content-pad); }
.fd-search-state :deep(.fd-skeleton) { max-width: 900px; }
.fd-search-results { padding: var(--fd-space-3) 0 var(--fd-space-8); }
.fd-search-group + .fd-search-group { margin-top: var(--fd-space-4); }
.fd-search-group h2 { display: flex; align-items: center; gap: var(--fd-space-2); margin: 0; padding: 8px var(--fd-content-pad); color: var(--fd-text-muted); font-size: var(--fd-text-11); font-weight: var(--fd-weight-semibold); text-transform: uppercase; }
.fd-search-group h2 span { padding: 1px 5px; border-radius: 10px; background: var(--fd-surface-raised); font-variant-numeric: tabular-nums; }
.fd-search-result { display: flex; align-items: center; justify-content: space-between; gap: var(--fd-space-3); width: 100%; min-height: 56px; padding: 8px var(--fd-content-pad); border: 0; border-bottom: 1px solid var(--fd-border-subtle); background: transparent; color: var(--fd-text); text-align: left; cursor: pointer; }
.fd-search-result:hover, .fd-search-result:focus-visible { background: var(--fd-surface-hover); outline: none; }
.fd-search-result__main { display: flex; flex-direction: column; gap: 2px; min-width: 0; }
.fd-search-result__main strong { overflow: hidden; font-size: var(--fd-text-13); font-weight: var(--fd-weight-medium); text-overflow: ellipsis; white-space: nowrap; }
.fd-search-result__main span, .fd-search-result__main small { overflow: hidden; color: var(--fd-text-muted); font-size: var(--fd-text-11); text-overflow: ellipsis; white-space: nowrap; }
.fd-search-result__open { color: var(--fd-text-muted); font-size: var(--fd-text-11); opacity: 0; }
.fd-search-result:hover .fd-search-result__open, .fd-search-result:focus-visible .fd-search-result__open { opacity: 1; }
</style>
