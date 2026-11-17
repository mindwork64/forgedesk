<script setup lang="ts">
import { ArrowLeft, Plus } from '@lucide/vue'
import { computed, onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRoute, useRouter } from 'vue-router'

import IssueList from '@/components/issues/IssueList.vue'
import PageHeader from '@/components/layout/PageHeader.vue'
import PlanningStatusBadge from '@/components/planning/PlanningStatusBadge.vue'
import ProgressBar from '@/components/planning/ProgressBar.vue'
import FdButton from '@/components/ui/FdButton.vue'
import FdConfirmDialog from '@/components/ui/FdConfirmDialog.vue'
import FdEmptyState from '@/components/ui/FdEmptyState.vue'
import FdErrorState from '@/components/ui/FdErrorState.vue'
import FdLoadingSkeleton from '@/components/ui/FdLoadingSkeleton.vue'
import FdSelect from '@/components/ui/FdSelect.vue'
import { api } from '@/lib/ipc'
import {
  formatDateRange,
  formatShortDate,
  milestoneStatusLabel,
} from '@/lib/planning'
import { useAppStore } from '@/stores/app'
import { usePlanningStore } from '@/stores/planning'
import type { Document, IssueSummary, Milestone } from '@/types/ipc'

import MilestoneDialog from './MilestoneDialog.vue'
import ProjectDialog from './ProjectDialog.vue'
import ProjectRepositories from './ProjectRepositories.vue'

type ProjectTab = 'overview' | 'issues' | 'roadmap' | 'documents' | 'repositories' | 'activity'

const route = useRoute()
const router = useRouter()
const planning = usePlanningStore()
const app = useAppStore()
const { t } = useI18n()

const activeTab = ref<ProjectTab>('overview')
const projectIssues = ref<IssueSummary[]>([])
const documents = ref<Document[]>([])
const editOpen = ref(false)
const milestoneOpen = ref(false)
const archiveDialogOpen = ref(false)
const archiveBusy = ref(false)
const issuesLoading = ref(false)

const projectId = computed(() => String(route.params.id ?? ''))
const project = computed(() => planning.project?.project ?? null)
const milestones = computed(() => planning.project?.milestones ?? [])

const tabs = computed(() => [
  { id: 'overview' as const, label: t('projects.tabs.overview') },
  { id: 'issues' as const, label: t('projects.tabs.issues') },
  { id: 'roadmap' as const, label: t('projects.tabs.roadmap') },
  { id: 'documents' as const, label: t('projects.tabs.documents') },
  { id: 'repositories' as const, label: t('projects.tabs.repositories') },
  { id: 'activity' as const, label: t('projects.tabs.activity') },
])

const description = computed(() => {
  if (!project.value) {
    return ''
  }
  const dates = formatDateRange(
    project.value.startDate,
    project.value.targetDate,
    app.resolvedLocale,
  )
  return [project.value.initiative?.name, dates].filter(Boolean).join(' · ')
})

const projectDocuments = computed(() =>
  documents.value.filter((document) => document.projectId === projectId.value),
)

async function loadIssues(): Promise<void> {
  issuesLoading.value = true
  try {
    projectIssues.value = await api.listIssues({
      filter: {
        kind: 'condition',
        field: 'project',
        operator: 'eq',
        value: projectId.value,
      },
      sort: [{ field: 'updatedAt', direction: 'desc' }],
      includeArchived: false,
      limit: 500,
    })
  } catch {
    projectIssues.value = []
  } finally {
    issuesLoading.value = false
  }
}

async function loadAll(): Promise<void> {
  if (!projectId.value) {
    return
  }
  await Promise.all([
    planning.loadProject(projectId.value),
    planning.loadInitiatives(),
    planning.loadDocuments(),
    loadIssues(),
  ])
  documents.value = planning.documents
}

watch(projectId, () => {
  activeTab.value = 'overview'
  void loadAll()
})

onMounted(() => {
  void loadAll()
})

function openAddIssue(): void {
  void router.push('/my-issues')
}

function openIssue(issueId: string): void {
  void router.push({ path: '/my-issues', query: { issue: issueId } })
}

function openDocument(document: Document): void {
  void router.push({ path: '/documents', query: { doc: document.id } })
}

async function linkIssue(milestone: Milestone, issueId: string): Promise<void> {
  if (!issueId) {
    return
  }
  const ok = await planning.assignMilestoneIssue(
    projectId.value,
    milestone.id,
    issueId,
    true,
  )
  if (ok) {
    await Promise.all([planning.loadProject(projectId.value), loadIssues()])
  }
}

function milestoneLabel(milestone: Milestone): string {
  return milestoneStatusLabel(milestone.status, t)
}

async function archiveCurrentProject(): Promise<void> {
  if (!project.value || archiveBusy.value) return
  archiveBusy.value = true
  const archived = await planning.archiveProject(projectId.value, true)
  archiveBusy.value = false
  archiveDialogOpen.value = false
  if (archived) void router.push('/projects')
}

async function restoreCurrentProject(): Promise<void> {
  if (!project.value || archiveBusy.value) return
  archiveBusy.value = true
  await planning.archiveProject(projectId.value, false)
  archiveBusy.value = false
}
</script>

<template>
  <div class="fd-project-page">
    <div class="fd-project-back">
      <FdButton variant="ghost" size="sm" @click="router.push('/projects')">
        <ArrowLeft :size="13" aria-hidden="true" />
        {{ t('pages.projects.title') }}
      </FdButton>
    </div>

    <div v-if="planning.loading && !project" class="fd-project-state">
      <FdLoadingSkeleton :rows="8" :height="42" />
    </div>
    <div v-else-if="planning.error" class="fd-project-state">
      <FdErrorState
        :title="t('projects.createError')"
        :message="planning.error.message"
        :code="planning.error.code"
        :detail="planning.error.detail"
        :retry-label="t('common.retry')"
        @retry="loadAll"
      />
    </div>
    <template v-else-if="project">
      <PageHeader :title="project.name" :description="description">
        <template #actions>
          <FdButton
            v-if="project.archivedAt"
            variant="secondary"
            size="md"
            :loading="archiveBusy"
            @click="restoreCurrentProject"
          >
            {{ t('projects.restoreProject') }}
          </FdButton>
          <FdButton
            v-else
            variant="destructive"
            size="md"
            @click="archiveDialogOpen = true"
          >
            {{ t('projects.archiveProject') }}
          </FdButton>
          <FdButton v-if="!project.archivedAt" variant="ghost" size="md" @click="editOpen = true">
            {{ t('projects.editProject') }}
          </FdButton>
          <FdButton v-if="!project.archivedAt" variant="primary" size="md" @click="openAddIssue">
            <Plus :size="14" aria-hidden="true" />
            {{ t('issues.newIssue') }}
          </FdButton>
        </template>
      </PageHeader>

      <div class="fd-project-meta">
        <PlanningStatusBadge :status="project.status" />
        <span class="fd-project-meta__dot" aria-hidden="true">·</span>
        <span>{{ t(`priority.${project.priority}`) }}</span>
        <span class="fd-project-meta__dot" aria-hidden="true">·</span>
        <span>{{ formatShortDate(project.targetDate, app.resolvedLocale) }}</span>
      </div>

      <nav class="fd-project-tabs" :aria-label="project.name">
        <button
          v-for="tab in tabs"
          :key="tab.id"
          type="button"
          class="fd-project-tab"
          :class="{ 'fd-project-tab--active': activeTab === tab.id }"
          :aria-current="activeTab === tab.id ? 'page' : undefined"
          @click="activeTab = tab.id"
        >
          {{ tab.label }}
        </button>
      </nav>

      <div class="fd-project-content">
        <template v-if="activeTab === 'overview'">
          <div class="fd-project-overview-grid">
            <section class="fd-project-section">
              <h2>{{ t('planning.description') }}</h2>
              <p v-if="project.description" class="fd-project-description">
                {{ project.description }}
              </p>
              <p v-else class="fd-project-muted">{{ t('projects.noDescription') }}</p>
            </section>

            <section class="fd-project-section">
              <h2>{{ t('planning.progress') }}</h2>
              <ProgressBar
                :percent="project.progress.percent"
                :completed="project.progress.completed"
                :total="project.progress.total"
              />
              <p class="fd-project-muted">{{ t('projects.progressMethod') }}</p>
            </section>

            <section class="fd-project-section">
              <div class="fd-project-section__head">
                <h2>{{ t('projects.milestones') }}</h2>
                <FdButton variant="ghost" size="sm" @click="milestoneOpen = true">
                  <Plus :size="12" aria-hidden="true" />
                  {{ t('projects.addMilestone') }}
                </FdButton>
              </div>
              <p v-if="milestones.length === 0" class="fd-project-muted">
                {{ t('projects.noMilestones') }}
              </p>
              <div v-else class="fd-milestone-list">
                <div v-for="milestone in milestones" :key="milestone.id" class="fd-milestone-row">
                  <div class="fd-milestone-row__main">
                    <strong>{{ milestone.name }}</strong>
                    <span>
                      {{ milestoneLabel(milestone) }}
                      <template v-if="milestone.dueDate">
                        · {{ formatShortDate(milestone.dueDate, app.resolvedLocale) }}
                      </template>
                    </span>
                  </div>
                  <div class="fd-milestone-row__progress">
                    <ProgressBar
                      :percent="milestone.progress.percent"
                      :completed="milestone.progress.completed"
                      :total="milestone.progress.total"
                    />
                  </div>
                  <FdSelect
                    :model-value="''"
                    class="fd-milestone-row__link"
                    :aria-label="t('projects.linkIssue')"
                    @update:model-value="linkIssue(milestone, $event)"
                  >
                    <option value="">{{ t('projects.linkIssue') }}</option>
                    <option
                      v-for="issue in projectIssues"
                      :key="issue.id"
                      :value="issue.id"
                    >
                      {{ issue.key }} · {{ issue.title }}
                    </option>
                  </FdSelect>
                </div>
              </div>
            </section>

            <section class="fd-project-section">
              <h2>{{ t('projects.issueCounts') }}</h2>
              <p v-if="!planning.project?.issueCounts.length" class="fd-project-muted">
                {{ t('projects.noIssues') }}
              </p>
              <div v-else class="fd-count-grid">
                <div
                  v-for="count in planning.project.issueCounts"
                  :key="count.status.id"
                  class="fd-count-row"
                >
                  <span>{{ count.status.name }}</span>
                  <strong>{{ count.count }}</strong>
                </div>
              </div>
            </section>

            <section class="fd-project-section">
              <h2>{{ t('projects.tabs.documents') }}</h2>
              <p v-if="projectDocuments.length === 0" class="fd-project-muted">
                {{ t('projects.noDocuments') }}
              </p>
              <button
                v-for="document in projectDocuments"
                :key="document.id"
                type="button"
                class="fd-project-doc-link"
                @click="openDocument(document)"
              >
                {{ document.title }}
              </button>
            </section>

            <section class="fd-project-section">
              <h2>{{ t('projects.tabs.repositories') }}</h2>
              <ProjectRepositories :project-id="projectId" />
            </section>

            <section class="fd-project-section">
              <h2>{{ t('projects.tabs.activity') }}</h2>
              <p class="fd-project-muted">{{ t('projects.noActivity') }}</p>
            </section>
          </div>
        </template>

        <template v-else-if="activeTab === 'issues'">
          <div v-if="issuesLoading" class="fd-project-state">
            <FdLoadingSkeleton :rows="6" :height="40" />
          </div>
          <FdEmptyState
            v-else-if="projectIssues.length === 0"
            :icon="Plus"
            :title="t('projects.noIssues')"
            :body="t('projects.emptyBody')"
          />
          <IssueList
            v-else
            :issues="projectIssues"
            :selected-ids="[]"
            @select="openIssue"
          />
        </template>

        <template v-else-if="activeTab === 'roadmap'">
          <div v-if="milestones.length === 0" class="fd-project-state">
            <FdEmptyState
              :icon="Plus"
              :title="t('projects.noMilestones')"
              :body="t('projects.addMilestone')"
            />
          </div>
          <div v-else class="fd-project-roadmap">
            <div v-for="milestone in milestones" :key="milestone.id" class="fd-project-roadmap__row">
              <span>{{ milestone.name }}</span>
              <span class="fd-project-roadmap__date">
                {{ milestone.dueDate ? formatShortDate(milestone.dueDate, app.resolvedLocale) : '—' }}
              </span>
              <ProgressBar
                :percent="milestone.progress.percent"
                :completed="milestone.progress.completed"
                :total="milestone.progress.total"
              />
            </div>
          </div>
        </template>

        <template v-else-if="activeTab === 'documents'">
          <div v-if="projectDocuments.length === 0" class="fd-project-state">
            <FdEmptyState
              :icon="Plus"
              :title="t('projects.noDocuments')"
              :body="t('documents.emptyBody')"
            />
          </div>
          <div v-else class="fd-project-doc-list">
            <button
              v-for="document in projectDocuments"
              :key="document.id"
              type="button"
              class="fd-project-doc-row"
              @click="openDocument(document)"
            >
              <strong>{{ document.title }}</strong>
              <span>{{ formatShortDate(document.updatedAt.slice(0, 10), app.resolvedLocale) }}</span>
            </button>
          </div>
        </template>

        <template v-else-if="activeTab === 'repositories'">
          <ProjectRepositories :project-id="projectId" />
        </template>

        <template v-else>
          <div class="fd-project-state">
            <FdEmptyState
              :icon="Plus"
              :title="t('projects.noActivity')"
              :body="t('projects.noActivity')"
            />
          </div>
        </template>
      </div>
    </template>

    <ProjectDialog
      v-if="project"
      v-model:open="editOpen"
      :project="project"
      :initiatives="planning.initiatives"
      @saved="planning.loadProject(projectId)"
    />
    <MilestoneDialog
      v-if="project"
      v-model:open="milestoneOpen"
      :project-id="projectId"
      @saved="planning.loadProject(projectId)"
    />
    <FdConfirmDialog
      v-model="archiveDialogOpen"
      :title="t('projects.archiveTitle')"
      :body="t('projects.archiveBody')"
      :confirm-label="t('projects.archiveConfirm')"
      :busy="archiveBusy"
      destructive
      @confirm="archiveCurrentProject"
    />
  </div>
</template>

<style scoped>
.fd-project-page {
  display: flex;
  flex-direction: column;
  width: 100%;
  height: 100%;
  min-width: 0;
  min-height: 0;
  overflow: hidden;
}

.fd-project-back {
  padding: 8px var(--fd-content-pad) 0;
}

.fd-project-state {
  display: flex;
  justify-content: center;
  padding: var(--fd-space-6) var(--fd-content-pad);
}

.fd-project-meta {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 0 var(--fd-content-pad) 10px;
  color: var(--fd-text-muted);
  font-size: var(--fd-text-11);
}

.fd-project-meta__dot {
  opacity: 0.5;
}

.fd-project-tabs {
  display: flex;
  flex: none;
  gap: 2px;
  padding: 0 var(--fd-content-pad);
  border-bottom: 1px solid var(--fd-border);
}

.fd-project-tab {
  height: 34px;
  padding: 0 10px;
  border: 0;
  border-bottom: 2px solid transparent;
  background: transparent;
  color: var(--fd-text-muted);
  font-size: var(--fd-text-12);
  cursor: pointer;
}

.fd-project-tab--active {
  border-bottom-color: var(--fd-accent);
  color: var(--fd-text-primary);
}

.fd-project-content {
  flex: 1;
  min-height: 0;
  overflow: auto;
}

.fd-project-overview-grid {
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
  gap: var(--fd-space-4) var(--fd-space-6);
  padding: var(--fd-space-4) var(--fd-content-pad) var(--fd-space-6);
}

.fd-project-section {
  min-width: 0;
  padding-bottom: var(--fd-space-4);
  border-bottom: 1px solid var(--fd-border-subtle);
}

.fd-project-section h2 {
  margin: 0 0 var(--fd-space-2);
  font-size: var(--fd-text-12);
  font-weight: var(--fd-weight-semibold);
}

.fd-project-section__head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--fd-space-2);
}

.fd-project-description {
  margin: 0;
  color: var(--fd-text-secondary);
  font-size: var(--fd-text-12);
  line-height: var(--fd-line-body);
  white-space: pre-wrap;
}

.fd-project-muted {
  margin: var(--fd-space-2) 0 0;
  color: var(--fd-text-muted);
  font-size: var(--fd-text-11);
  line-height: var(--fd-line-body);
}

.fd-milestone-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.fd-milestone-row {
  display: grid;
  grid-template-columns: minmax(140px, 1.1fr) minmax(100px, 0.8fr) minmax(120px, 1fr);
  align-items: center;
  gap: var(--fd-space-2);
  min-height: 38px;
  padding: 4px 0;
  border-bottom: 1px solid var(--fd-border-subtle);
}

.fd-milestone-row__main {
  display: flex;
  flex-direction: column;
  gap: 1px;
  min-width: 0;
}

.fd-milestone-row__main strong {
  overflow: hidden;
  font-size: var(--fd-text-12);
  font-weight: var(--fd-weight-medium);
  text-overflow: ellipsis;
  white-space: nowrap;
}

.fd-milestone-row__main span {
  color: var(--fd-text-muted);
  font-size: var(--fd-text-11);
}

.fd-milestone-row__progress {
  min-width: 0;
}

.fd-milestone-row__link {
  width: 100%;
}

.fd-count-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 4px 12px;
}

.fd-count-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  min-height: 26px;
  color: var(--fd-text-secondary);
  font-size: var(--fd-text-11);
}

.fd-count-row strong {
  color: var(--fd-text-primary);
  font-variant-numeric: tabular-nums;
}

.fd-project-doc-link,
.fd-project-doc-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  width: 100%;
  min-height: 30px;
  padding: 0 4px;
  border: 0;
  border-bottom: 1px solid var(--fd-border-subtle);
  background: transparent;
  color: var(--fd-text-secondary);
  font-size: var(--fd-text-12);
  text-align: left;
  cursor: pointer;
}

.fd-project-doc-link:hover,
.fd-project-doc-row:hover {
  color: var(--fd-accent);
}

.fd-project-doc-list {
  padding: var(--fd-space-3) var(--fd-content-pad);
}

.fd-project-doc-row span {
  color: var(--fd-text-muted);
  font-size: var(--fd-text-11);
}

.fd-project-roadmap {
  padding: var(--fd-space-4) var(--fd-content-pad);
}

.fd-project-roadmap__row {
  display: grid;
  grid-template-columns: minmax(180px, 1fr) 100px minmax(160px, 1fr);
  align-items: center;
  gap: var(--fd-space-3);
  min-height: 40px;
  border-bottom: 1px solid var(--fd-border-subtle);
  font-size: var(--fd-text-12);
}

.fd-project-roadmap__date {
  color: var(--fd-text-muted);
  font-size: var(--fd-text-11);
}

@media (max-width: 1199px) {
  .fd-project-overview-grid {
    grid-template-columns: minmax(0, 1fr);
  }
}

@media (max-width: 760px) {
  .fd-milestone-row {
    grid-template-columns: 1fr;
  }
}
</style>
