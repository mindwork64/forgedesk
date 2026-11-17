<script setup lang="ts">
import { Search as SearchIcon } from '@lucide/vue'
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
import type { Document, Initiative, IssueSummary, Project, Repository } from '@/types/ipc'

const { t } = useI18n()
const router = useRouter()

const query = ref('')
const issues = ref<IssueSummary[]>([])
const projects = ref<Project[]>([])
const initiatives = ref<Initiative[]>([])
const documents = ref<Document[]>([])
const repositories = ref<Repository[]>([])
const loadingReferenceData = ref(true)
const searching = ref(false)
const error = ref<string | null>(null)

let requestId = 0
let searchTimer: ReturnType<typeof setTimeout> | null = null

const needle = computed(() => query.value.trim().toLocaleLowerCase())

function matches(value: string | null | undefined): boolean {
  return (value ?? '').toLocaleLowerCase().includes(needle.value)
}

const matchingInitiatives = computed(() =>
  initiatives.value
    .filter((item) => matches(item.name) || matches(item.description))
    .slice(0, 8),
)
const matchingDocuments = computed(() =>
  documents.value
    .filter((item) => !item.archivedAt && (matches(item.title) || matches(item.body)))
    .slice(0, 8),
)
const matchingRepositories = computed(() =>
  repositories.value
    .filter((item) =>
      !item.isArchived &&
      (matches(item.fullName) || matches(item.description) || matches(item.host)),
    )
    .slice(0, 8),
)
const resultCount = computed(
  () =>
    issues.value.length +
    projects.value.length +
    matchingInitiatives.value.length +
    matchingDocuments.value.length +
    matchingRepositories.value.length,
)

async function loadReferenceData(): Promise<void> {
  loadingReferenceData.value = true
  error.value = null
  try {
    const [nextInitiatives, nextDocuments, nextRepositories] = await Promise.all([
      api.listInitiatives(),
      api.listDocuments(),
      api.listRepositories(),
    ])
    initiatives.value = nextInitiatives
    documents.value = nextDocuments
    repositories.value = nextRepositories
  } catch (caught) {
    error.value = toIpcError(caught).message
  } finally {
    loadingReferenceData.value = false
  }
}

async function runSearch(value: string, id: number): Promise<void> {
  searching.value = true
  error.value = null
  try {
    const [nextIssues, nextProjects] = await Promise.all([
      api.listIssues({
        search: value,
        sort: [{ field: 'updatedAt', direction: 'desc' }],
        includeArchived: false,
        limit: 12,
      }),
      api.listProjectCards({ search: value, includeArchived: false }),
    ])
    if (id === requestId) {
      issues.value = nextIssues
      projects.value = nextProjects.slice(0, 8)
    }
  } catch (caught) {
    if (id === requestId) {
      error.value = toIpcError(caught).message
      issues.value = []
      projects.value = []
    }
  } finally {
    if (id === requestId) searching.value = false
  }
}

watch(needle, (value) => {
  requestId += 1
  const id = requestId
  if (searchTimer) {
    clearTimeout(searchTimer)
    searchTimer = null
  }
  if (value.length < 2) {
    issues.value = []
    projects.value = []
    searching.value = false
    error.value = null
    return
  }
  searching.value = true
  searchTimer = setTimeout(() => {
    void runSearch(value, id)
  }, 180)
})

onMounted(() => {
  void loadReferenceData()
})

onBeforeUnmount(() => {
  requestId += 1
  if (searchTimer) clearTimeout(searchTimer)
})

function openIssue(issue: IssueSummary): void {
  void router.push({ path: '/my-issues', query: { issue: issue.id } })
}
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
          :placeholder="t('searchPage.placeholder')"
          :aria-label="t('searchPage.placeholder')"
        />
      </template>
      <template #right>
        <span class="fd-search-scope">{{ t('searchPage.localScope') }}</span>
      </template>
    </PageToolbar>

    <main class="fd-search-scroll">
      <div v-if="loadingReferenceData || searching" class="fd-search-state">
        <FdLoadingSkeleton :rows="6" :height="48" />
      </div>
      <div v-else-if="error" class="fd-search-state">
        <FdErrorState
          :title="t('searchPage.loadError')"
          :message="error"
          :retry-label="t('common.retry')"
          @retry="loadReferenceData"
        />
      </div>
      <div v-else-if="needle.length < 2" class="fd-search-state">
        <FdEmptyState :icon="SearchIcon" :title="t('searchPage.minChars')" />
      </div>
      <div v-else-if="resultCount === 0" class="fd-search-state">
        <FdEmptyState
          :icon="SearchIcon"
          :title="t('searchPage.noResults')"
          :body="t('searchPage.noResultsBody')"
        />
      </div>
      <div v-else class="fd-search-results">
        <section v-if="issues.length">
          <h2>{{ t('searchPage.issues') }} <span>{{ issues.length }}</span></h2>
          <button
            v-for="issue in issues"
            :key="issue.id"
            class="fd-search-hit"
            type="button"
            @click="openIssue(issue)"
          >
            <strong><span class="fd-search-key">{{ issue.key }}</span> {{ issue.title }}</strong>
            <small>{{ t('searchPage.issueStatus', { status: issue.status.name }) }}</small>
          </button>
        </section>

        <section v-if="projects.length">
          <h2>{{ t('searchPage.projects') }} <span>{{ projects.length }}</span></h2>
          <button
            v-for="project in projects"
            :key="project.id"
            class="fd-search-hit"
            type="button"
            @click="router.push(`/projects/${project.id}`)"
          >
            <strong>{{ project.name }}</strong>
            <small>{{ project.description || project.status }}</small>
          </button>
        </section>

        <section v-if="matchingInitiatives.length">
          <h2>{{ t('searchPage.initiatives') }} <span>{{ matchingInitiatives.length }}</span></h2>
          <button
            v-for="initiative in matchingInitiatives"
            :key="initiative.id"
            class="fd-search-hit"
            type="button"
            @click="router.push(`/initiatives/${initiative.id}`)"
          >
            <strong>{{ initiative.name }}</strong>
            <small>{{ initiative.description || initiative.status }}</small>
          </button>
        </section>

        <section v-if="matchingDocuments.length">
          <h2>{{ t('searchPage.documents') }} <span>{{ matchingDocuments.length }}</span></h2>
          <button
            v-for="document in matchingDocuments"
            :key="document.id"
            class="fd-search-hit"
            type="button"
            @click="router.push({ path: '/documents', query: { doc: document.id } })"
          >
            <strong>{{ document.title }}</strong>
            <small>{{ document.projectName || document.initiativeName || t('searchPage.documents') }}</small>
          </button>
        </section>

        <section v-if="matchingRepositories.length">
          <h2>{{ t('searchPage.repositories') }} <span>{{ matchingRepositories.length }}</span></h2>
          <button
            v-for="repository in matchingRepositories"
            :key="repository.id"
            class="fd-search-hit"
            type="button"
            @click="router.push(`/repositories/${repository.id}`)"
          >
            <strong>{{ repository.fullName }}</strong>
            <small>{{ repository.description || repository.host }}</small>
          </button>
        </section>
      </div>
    </main>
  </div>
</template>

<style scoped>
.fd-search-page {
  display: flex;
  flex-direction: column;
  width: 100%;
  height: 100%;
  min-width: 0;
  min-height: 0;
  overflow: hidden;
}
.fd-search-input { width: min(560px, 60vw); }
.fd-search-scope { color: var(--fd-text-muted); font-size: var(--fd-text-11); }
.fd-search-scroll { flex: 1; min-height: 0; overflow: auto; }
.fd-search-state { padding: var(--fd-space-6) var(--fd-content-pad); }
.fd-search-results {
  display: flex;
  flex-direction: column;
  gap: var(--fd-space-5);
  max-width: 920px;
  padding: var(--fd-space-4) var(--fd-content-pad) var(--fd-space-8);
}
.fd-search-results section { display: flex; flex-direction: column; gap: 2px; }
.fd-search-results h2 {
  display: flex;
  justify-content: space-between;
  margin: 0 0 var(--fd-space-2);
  color: var(--fd-text-muted);
  font-size: var(--fd-text-11);
  font-weight: var(--fd-weight-medium);
  text-transform: uppercase;
}
.fd-search-results h2 span { font-variant-numeric: tabular-nums; }
.fd-search-hit {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 3px;
  width: 100%;
  min-height: 48px;
  padding: 8px 10px;
  border: 1px solid transparent;
  border-radius: var(--fd-radius-md);
  background: transparent;
  color: var(--fd-text-primary);
  text-align: left;
  cursor: pointer;
}
.fd-search-hit:hover, .fd-search-hit:focus-visible {
  border-color: var(--fd-border-subtle);
  background: var(--fd-surface-hover);
  outline: none;
}
.fd-search-hit strong { font-size: var(--fd-text-13); font-weight: var(--fd-weight-medium); }
.fd-search-hit small { color: var(--fd-text-muted); font-size: var(--fd-text-11); }
.fd-search-key { color: var(--fd-text-muted); font-family: var(--fd-font-mono); }
</style>
