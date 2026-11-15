<script setup lang="ts">
import { FolderKanban, Plus } from '@lucide/vue'
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRouter } from 'vue-router'

import PageHeader from '@/components/layout/PageHeader.vue'
import PageToolbar from '@/components/layout/PageToolbar.vue'
import PriorityIndicator from '@/components/issues/PriorityIndicator.vue'
import PlanningStatusBadge from '@/components/planning/PlanningStatusBadge.vue'
import ProgressBar from '@/components/planning/ProgressBar.vue'
import FdButton from '@/components/ui/FdButton.vue'
import FdEmptyState from '@/components/ui/FdEmptyState.vue'
import FdErrorState from '@/components/ui/FdErrorState.vue'
import FdInput from '@/components/ui/FdInput.vue'
import FdLoadingSkeleton from '@/components/ui/FdLoadingSkeleton.vue'
import FdSelect from '@/components/ui/FdSelect.vue'
import { formatRelative } from '@/lib/format'
import {
  PLANNING_STATUSES,
  formatShortDate,
  planningStatusLabel,
} from '@/lib/planning'
import { useAppStore } from '@/stores/app'
import { usePlanningStore } from '@/stores/planning'
import type { PlanningStatus, Project } from '@/types/ipc'

import ProjectDialog from './ProjectDialog.vue'

const planning = usePlanningStore()
const app = useAppStore()
const router = useRouter()
const { t } = useI18n()

const search = ref('')
const status = ref<PlanningStatus | ''>('')
const initiativeId = ref('')
const includeArchived = ref(false)
const dialogOpen = ref(false)
const editing = ref<Project | null>(null)
let searchTimer: ReturnType<typeof setTimeout> | null = null

const statusOptions = PLANNING_STATUSES.map((value) => ({
  value,
  label: planningStatusLabel(value, t),
}))

const hasProjects = computed(() => planning.projects.length > 0)

async function load(): Promise<void> {
  await planning.loadProjects({
    search: search.value.trim() || null,
    status: status.value || null,
    initiativeId: initiativeId.value || null,
    includeArchived: includeArchived.value,
  })
}

watch(search, () => {
  if (searchTimer) {
    clearTimeout(searchTimer)
  }
  searchTimer = setTimeout(() => {
    void load()
  }, 200)
})

watch([status, initiativeId, includeArchived], () => {
  void load()
})

onMounted(async () => {
  await Promise.all([load(), planning.loadInitiatives()])
})

onBeforeUnmount(() => {
  if (searchTimer) {
    clearTimeout(searchTimer)
  }
})

function openCreate(): void {
  editing.value = null
  dialogOpen.value = true
}

function openProject(project: Project): void {
  void router.push(`/projects/${project.id}`)
}

function onSaved(project: Project): void {
  void router.push(`/projects/${project.id}`)
}
</script>

<template>
  <div class="fd-projects-page">
    <PageHeader
      :title="t('pages.projects.title')"
      :description="t('pages.projects.description')"
    >
      <template #actions>
        <FdButton variant="primary" @click="openCreate">
          <Plus :size="14" aria-hidden="true" />
          {{ t('projects.newProject') }}
        </FdButton>
      </template>
    </PageHeader>

    <PageToolbar>
      <template #left>
        <FdInput
          v-model="search"
          type="search"
          class="fd-projects-search"
          :placeholder="t('projects.searchPlaceholder')"
          :aria-label="t('projects.searchPlaceholder')"
        />
      </template>
      <template #right>
        <FdSelect
          v-model="status"
          class="fd-projects-filter"
          :aria-label="t('planning.statusLabel')"
        >
          <option value="">{{ t('projects.allStatuses') }}</option>
          <option
            v-for="option in statusOptions"
            :key="option.value"
            :value="option.value"
          >
            {{ option.label }}
          </option>
        </FdSelect>
        <FdSelect
          v-model="initiativeId"
          class="fd-projects-filter"
          :aria-label="t('projects.initiative')"
        >
          <option value="">{{ t('projects.allInitiatives') }}</option>
          <option
            v-for="initiative in planning.initiatives"
            :key="initiative.id"
            :value="initiative.id"
          >
            {{ initiative.name }}
          </option>
        </FdSelect>
        <FdButton
          :variant="includeArchived ? 'secondary' : 'ghost'"
          size="sm"
          @click="includeArchived = !includeArchived"
        >
          {{ includeArchived ? t('projects.hideArchived') : t('projects.showArchived') }}
        </FdButton>
      </template>
    </PageToolbar>

    <div class="fd-projects-scroll">
      <div v-if="planning.loading" class="fd-projects-state">
        <FdLoadingSkeleton :rows="8" :height="44" />
      </div>
      <div v-else-if="planning.error" class="fd-projects-state">
        <FdErrorState
          :title="t('projects.createError')"
          :message="planning.error.message"
          :code="planning.error.code"
          :detail="planning.error.detail"
          :retry-label="t('common.retry')"
          @retry="load"
        />
      </div>
      <div v-else-if="!hasProjects" class="fd-projects-state">
        <FdEmptyState
          :icon="FolderKanban"
          :title="t('projects.emptyTitle')"
          :body="t('projects.emptyBody')"
        >
          <FdButton variant="primary" @click="openCreate">
            <Plus :size="14" aria-hidden="true" />
            {{ t('projects.createFirst') }}
          </FdButton>
        </FdEmptyState>
      </div>
      <div v-else class="fd-project-table" role="table" :aria-label="t('pages.projects.title')">
        <div class="fd-project-grid fd-project-grid--head" role="row">
          <span role="columnheader">{{ t('projects.columns.name') }}</span>
          <span role="columnheader">{{ t('projects.columns.status') }}</span>
          <span role="columnheader">{{ t('projects.columns.progress') }}</span>
          <span role="columnheader">{{ t('projects.columns.priority') }}</span>
          <span role="columnheader">{{ t('projects.columns.targetDate') }}</span>
          <span role="columnheader">{{ t('projects.columns.repositories') }}</span>
          <span role="columnheader">{{ t('projects.columns.updated') }}</span>
        </div>
        <div
          v-for="project in planning.projects"
          :key="project.id"
          class="fd-project-grid fd-project-row"
          role="row"
          tabindex="0"
          :aria-label="project.name"
          @click="openProject(project)"
          @keydown.enter="openProject(project)"
        >
          <span class="fd-project-row__name" role="cell">
            <strong>{{ project.name }}</strong>
            <small>
              {{ project.initiative?.name ?? t('planning.noInitiative') }}
            </small>
          </span>
          <span role="cell"><PlanningStatusBadge :status="project.status" /></span>
          <span class="fd-project-row__progress" role="cell">
            <ProgressBar
              :percent="project.progress.percent"
              :completed="project.progress.completed"
              :total="project.progress.total"
            />
          </span>
          <span role="cell">
            <PriorityIndicator :priority="project.priority" show-label />
          </span>
          <span role="cell">
            {{ project.targetDate ? formatShortDate(project.targetDate, app.resolvedLocale) : '—' }}
          </span>
          <span class="fd-project-row__muted" role="cell">—</span>
          <span class="fd-project-row__muted" role="cell">
            {{ formatRelative(project.updatedAt, app.resolvedLocale) }}
          </span>
        </div>
      </div>
    </div>

    <ProjectDialog
      v-model:open="dialogOpen"
      :project="editing"
      :initiatives="planning.initiatives"
      @saved="onSaved"
    />
  </div>
</template>

<style scoped>
.fd-projects-page {
  display: flex;
  flex-direction: column;
  width: 100%;
  height: 100%;
  min-width: 0;
  min-height: 0;
  overflow: hidden;
}

.fd-projects-search {
  width: 260px;
}

.fd-projects-filter {
  width: 170px;
}

.fd-projects-scroll {
  flex: 1;
  min-height: 0;
  overflow: auto;
}

.fd-projects-state {
  display: flex;
  justify-content: center;
  padding: var(--fd-space-6) var(--fd-content-pad);
}

.fd-project-grid {
  display: grid;
  grid-template-columns: minmax(240px, 1.6fr) 120px minmax(180px, 1fr) 100px 120px 120px 110px;
  gap: var(--fd-space-3);
  align-items: center;
  min-height: 48px;
  padding: 0 var(--fd-content-pad);
}

.fd-project-grid--head {
  position: sticky;
  top: 0;
  z-index: 2;
  min-height: 36px;
  border-bottom: 1px solid var(--fd-border);
  background: var(--fd-bg);
  color: var(--fd-text-muted);
  font-size: var(--fd-text-11);
}

.fd-project-row {
  border-bottom: 1px solid var(--fd-border-subtle);
  cursor: pointer;
}

.fd-project-row:hover {
  background: var(--fd-surface-hover);
}

.fd-project-row__name {
  display: flex;
  flex-direction: column;
  gap: 1px;
  min-width: 0;
}

.fd-project-row__name strong {
  overflow: hidden;
  font-size: var(--fd-text-12);
  font-weight: var(--fd-weight-medium);
  white-space: nowrap;
  text-overflow: ellipsis;
}

.fd-project-row__name small {
  overflow: hidden;
  color: var(--fd-text-muted);
  font-size: var(--fd-text-11);
  white-space: nowrap;
  text-overflow: ellipsis;
}

.fd-project-row__progress {
  min-width: 0;
}

.fd-project-row__muted {
  color: var(--fd-text-muted);
  font-size: var(--fd-text-11);
}

@media (max-width: 1199px) {
  .fd-project-grid {
    grid-template-columns: minmax(220px, 1.6fr) 110px minmax(160px, 1fr) 90px 110px 100px;
  }

  .fd-project-grid > :nth-child(6) {
    display: none;
  }

  .fd-projects-search {
    width: 190px;
  }

  .fd-projects-filter {
    width: 150px;
  }
}
</style>
