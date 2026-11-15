<script setup lang="ts">
import { ArrowLeft, Target } from '@lucide/vue'
import { computed, onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRoute, useRouter } from 'vue-router'

import PageHeader from '@/components/layout/PageHeader.vue'
import PlanningStatusBadge from '@/components/planning/PlanningStatusBadge.vue'
import ProgressBar from '@/components/planning/ProgressBar.vue'
import FdButton from '@/components/ui/FdButton.vue'
import FdEmptyState from '@/components/ui/FdEmptyState.vue'
import FdErrorState from '@/components/ui/FdErrorState.vue'
import FdLoadingSkeleton from '@/components/ui/FdLoadingSkeleton.vue'
import { formatShortDate } from '@/lib/planning'
import { useAppStore } from '@/stores/app'
import { usePlanningStore } from '@/stores/planning'
import type { Initiative, Project } from '@/types/ipc'

import InitiativeDialog from './InitiativeDialog.vue'

const route = useRoute()
const router = useRouter()
const planning = usePlanningStore()
const app = useAppStore()
const { t } = useI18n()

const editOpen = ref(false)
const initiativeId = computed(() => String(route.params.id ?? ''))
const initiative = computed(() => planning.initiative?.initiative ?? null)
const projects = computed(() => planning.initiative?.projects ?? [])

const description = computed(() => {
  if (!initiative.value) {
    return ''
  }
  const dates = [initiative.value.startDate, initiative.value.targetDate]
    .filter(Boolean)
    .map((value) => formatShortDate(value as string, app.resolvedLocale))
    .join(' – ')
  return dates
})

async function load(): Promise<void> {
  if (initiativeId.value) {
    await planning.loadInitiative(initiativeId.value)
  }
}

onMounted(load)
watch(initiativeId, load)

function openProject(project: Project): void {
  void router.push(`/projects/${project.id}`)
}

function onSaved(saved: Initiative): void {
  void planning.loadInitiative(saved.id)
}
</script>

<template>
  <div class="fd-initiative-page">
    <div class="fd-initiative-back">
      <FdButton variant="ghost" size="sm" @click="router.push('/initiatives')">
        <ArrowLeft :size="13" aria-hidden="true" />
        {{ t('pages.initiatives.title') }}
      </FdButton>
    </div>

    <div v-if="planning.loading && !initiative" class="fd-initiative-state">
      <FdLoadingSkeleton :rows="7" :height="42" />
    </div>
    <div v-else-if="planning.error" class="fd-initiative-state">
      <FdErrorState
        :title="t('initiatives.createError')"
        :message="planning.error.message"
        :code="planning.error.code"
        :detail="planning.error.detail"
        :retry-label="t('common.retry')"
        @retry="load"
      />
    </div>
    <template v-else-if="initiative">
      <PageHeader :title="initiative.name" :description="description">
        <template #actions>
          <FdButton variant="ghost" size="md" @click="editOpen = true">
            {{ t('initiatives.editInitiative') }}
          </FdButton>
        </template>
      </PageHeader>

      <div class="fd-initiative-meta">
        <PlanningStatusBadge :status="initiative.status" />
        <span aria-hidden="true">·</span>
        <span>{{ t(`priority.${initiative.priority}`) }}</span>
        <span aria-hidden="true">·</span>
        <span>{{ t('planning.projectCount', { count: initiative.projectCount }) }}</span>
      </div>

      <div class="fd-initiative-content">
        <section class="fd-initiative-section">
          <h2>{{ t('planning.description') }}</h2>
          <p v-if="initiative.description" class="fd-initiative-description">
            {{ initiative.description }}
          </p>
          <p v-else class="fd-initiative-muted">{{ t('initiatives.noProjects') }}</p>
        </section>

        <section class="fd-initiative-section">
          <h2>{{ t('planning.progress') }}</h2>
          <ProgressBar
            :percent="initiative.progress.percent"
            :completed="initiative.progress.completed"
            :total="initiative.progress.total"
          />
          <p class="fd-initiative-muted">{{ t('initiatives.progressMethod') }}</p>
        </section>

        <section class="fd-initiative-section">
          <h2>{{ t('initiatives.projects') }}</h2>
          <p v-if="projects.length === 0" class="fd-initiative-muted">
            {{ t('initiatives.noProjects') }}
          </p>
          <div v-else class="fd-initiative-projects">
            <div class="fd-initiative-project-grid fd-initiative-project-grid--head">
              <span>{{ t('initiatives.columns.name') }}</span>
              <span>{{ t('initiatives.columns.status') }}</span>
              <span>{{ t('initiatives.columns.progress') }}</span>
              <span>{{ t('initiatives.columns.targetDate') }}</span>
            </div>
            <div
              v-for="project in projects"
              :key="project.id"
              class="fd-initiative-project-grid fd-initiative-project-row"
              tabindex="0"
              role="button"
              @click="openProject(project)"
              @keydown.enter="openProject(project)"
            >
              <strong>{{ project.name }}</strong>
              <PlanningStatusBadge :status="project.status" />
              <ProgressBar
                :percent="project.progress.percent"
                :completed="project.progress.completed"
                :total="project.progress.total"
              />
              <span>{{ project.targetDate ? formatShortDate(project.targetDate, app.resolvedLocale) : '—' }}</span>
            </div>
          </div>
        </section>
      </div>
    </template>

    <div v-else class="fd-initiative-state">
      <FdEmptyState
        :icon="Target"
        :title="t('initiatives.emptyTitle')"
        :body="t('initiatives.emptyBody')"
      />
    </div>

    <InitiativeDialog
      v-if="initiative"
      v-model:open="editOpen"
      :initiative="initiative"
      @saved="onSaved"
    />
  </div>
</template>

<style scoped>
.fd-initiative-page {
  display: flex;
  flex-direction: column;
  width: 100%;
  height: 100%;
  min-width: 0;
  min-height: 0;
  overflow: auto;
}

.fd-initiative-back {
  padding: 8px var(--fd-content-pad) 0;
}

.fd-initiative-state {
  display: flex;
  justify-content: center;
  padding: var(--fd-space-6) var(--fd-content-pad);
}

.fd-initiative-meta {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 0 var(--fd-content-pad) var(--fd-space-2);
  color: var(--fd-text-muted);
  font-size: var(--fd-text-11);
}

.fd-initiative-content {
  padding: 0 var(--fd-content-pad) var(--fd-space-6);
}

.fd-initiative-section {
  padding: var(--fd-space-4) 0;
  border-bottom: 1px solid var(--fd-border-subtle);
}

.fd-initiative-section h2 {
  margin: 0 0 var(--fd-space-2);
  font-size: var(--fd-text-12);
  font-weight: var(--fd-weight-semibold);
}

.fd-initiative-description {
  margin: 0;
  color: var(--fd-text-secondary);
  font-size: var(--fd-text-12);
  line-height: var(--fd-line-body);
  white-space: pre-wrap;
}

.fd-initiative-muted {
  margin: var(--fd-space-2) 0 0;
  color: var(--fd-text-muted);
  font-size: var(--fd-text-11);
}

.fd-initiative-project-grid {
  display: grid;
  grid-template-columns: minmax(220px, 1.5fr) 120px minmax(180px, 1fr) 120px;
  gap: var(--fd-space-3);
  align-items: center;
  min-height: 42px;
  border-bottom: 1px solid var(--fd-border-subtle);
  font-size: var(--fd-text-12);
}

.fd-initiative-project-grid--head {
  min-height: 32px;
  color: var(--fd-text-muted);
  font-size: var(--fd-text-11);
}

.fd-initiative-project-row {
  cursor: pointer;
}

.fd-initiative-project-row:hover {
  background: var(--fd-surface-hover);
}

.fd-initiative-project-row strong {
  font-weight: var(--fd-weight-medium);
}
</style>
