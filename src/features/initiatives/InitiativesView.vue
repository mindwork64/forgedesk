<script setup lang="ts">
import { Plus, Target } from '@lucide/vue'
import { computed, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRouter } from 'vue-router'

import PageHeader from '@/components/layout/PageHeader.vue'
import PageToolbar from '@/components/layout/PageToolbar.vue'
import PlanningStatusBadge from '@/components/planning/PlanningStatusBadge.vue'
import ProgressBar from '@/components/planning/ProgressBar.vue'
import FdButton from '@/components/ui/FdButton.vue'
import FdEmptyState from '@/components/ui/FdEmptyState.vue'
import FdErrorState from '@/components/ui/FdErrorState.vue'
import FdInput from '@/components/ui/FdInput.vue'
import FdLoadingSkeleton from '@/components/ui/FdLoadingSkeleton.vue'
import { formatRelative } from '@/lib/format'
import { formatShortDate } from '@/lib/planning'
import { useAppStore } from '@/stores/app'
import { usePlanningStore } from '@/stores/planning'
import type { Initiative } from '@/types/ipc'

import InitiativeDialog from './InitiativeDialog.vue'

const planning = usePlanningStore()
const app = useAppStore()
const router = useRouter()
const { t } = useI18n()

const search = ref('')
const dialogOpen = ref(false)
const filtered = computed(() => {
  const needle = search.value.trim().toLowerCase()
  if (!needle) {
    return planning.initiatives
  }
  return planning.initiatives.filter((initiative) =>
    initiative.name.toLowerCase().includes(needle),
  )
})

onMounted(() => {
  void planning.loadInitiatives()
})

function openInitiative(initiative: Initiative): void {
  void router.push(`/initiatives/${initiative.id}`)
}

function onSaved(initiative: Initiative): void {
  void router.push(`/initiatives/${initiative.id}`)
}
</script>

<template>
  <div class="fd-initiatives-page">
    <PageHeader
      :title="t('pages.initiatives.title')"
      :description="t('pages.initiatives.description')"
    >
      <template #actions>
        <FdButton variant="primary" @click="dialogOpen = true">
          <Plus :size="14" aria-hidden="true" />
          {{ t('initiatives.newInitiative') }}
        </FdButton>
      </template>
    </PageHeader>

    <PageToolbar>
      <template #left>
        <FdInput
          v-model="search"
          type="search"
          class="fd-initiatives-search"
          :placeholder="t('initiatives.searchPlaceholder')"
          :aria-label="t('initiatives.searchPlaceholder')"
        />
      </template>
    </PageToolbar>

    <div class="fd-initiatives-scroll">
      <div v-if="planning.loading" class="fd-initiatives-state">
        <FdLoadingSkeleton :rows="7" :height="42" />
      </div>
      <div v-else-if="planning.error" class="fd-initiatives-state">
        <FdErrorState
          :title="t('initiatives.createError')"
          :message="planning.error.message"
          :code="planning.error.code"
          :detail="planning.error.detail"
          :retry-label="t('common.retry')"
          @retry="planning.loadInitiatives()"
        />
      </div>
      <div v-else-if="filtered.length === 0" class="fd-initiatives-state">
        <FdEmptyState
          :icon="Target"
          :title="t('initiatives.emptyTitle')"
          :body="t('initiatives.emptyBody')"
        >
          <FdButton variant="primary" @click="dialogOpen = true">
            <Plus :size="14" aria-hidden="true" />
            {{ t('initiatives.createFirst') }}
          </FdButton>
        </FdEmptyState>
      </div>
      <div v-else class="fd-initiative-table" role="table" :aria-label="t('pages.initiatives.title')">
        <div class="fd-initiative-grid fd-initiative-grid--head" role="row">
          <span role="columnheader">{{ t('initiatives.columns.name') }}</span>
          <span role="columnheader">{{ t('initiatives.columns.status') }}</span>
          <span role="columnheader">{{ t('initiatives.columns.progress') }}</span>
          <span role="columnheader">{{ t('initiatives.columns.targetDate') }}</span>
          <span role="columnheader">{{ t('initiatives.columns.projects') }}</span>
          <span role="columnheader">{{ t('initiatives.columns.updated') }}</span>
        </div>
        <div
          v-for="initiative in filtered"
          :key="initiative.id"
          class="fd-initiative-grid fd-initiative-row"
          role="row"
          tabindex="0"
          :aria-label="initiative.name"
          @click="openInitiative(initiative)"
          @keydown.enter="openInitiative(initiative)"
        >
          <span class="fd-initiative-row__name" role="cell">
            <strong>{{ initiative.name }}</strong>
            <small>{{ initiative.description || t('planning.noInitiative') }}</small>
          </span>
          <span role="cell"><PlanningStatusBadge :status="initiative.status" /></span>
          <span class="fd-initiative-row__progress" role="cell">
            <ProgressBar
              :percent="initiative.progress.percent"
              :completed="initiative.progress.completed"
              :total="initiative.progress.total"
            />
          </span>
          <span role="cell">
            {{ initiative.targetDate ? formatShortDate(initiative.targetDate, app.resolvedLocale) : '—' }}
          </span>
          <span role="cell">{{ initiative.projectCount }}</span>
          <span class="fd-initiative-row__muted" role="cell">
            {{ formatRelative(initiative.updatedAt, app.resolvedLocale) }}
          </span>
        </div>
      </div>
    </div>

    <InitiativeDialog v-model:open="dialogOpen" @saved="onSaved" />
  </div>
</template>

<style scoped>
.fd-initiatives-page {
  display: flex;
  flex-direction: column;
  width: 100%;
  height: 100%;
  min-width: 0;
  min-height: 0;
  overflow: hidden;
}

.fd-initiatives-search {
  width: 280px;
}

.fd-initiatives-scroll {
  flex: 1;
  min-height: 0;
  overflow: auto;
}

.fd-initiatives-state {
  display: flex;
  justify-content: center;
  padding: var(--fd-space-6) var(--fd-content-pad);
}

.fd-initiative-grid {
  display: grid;
  grid-template-columns: minmax(260px, 1.7fr) 120px minmax(180px, 1fr) 120px 90px 110px;
  gap: var(--fd-space-3);
  align-items: center;
  min-height: 48px;
  padding: 0 var(--fd-content-pad);
}

.fd-initiative-grid--head {
  position: sticky;
  top: 0;
  z-index: 2;
  min-height: 36px;
  border-bottom: 1px solid var(--fd-border);
  background: var(--fd-bg);
  color: var(--fd-text-muted);
  font-size: var(--fd-text-11);
}

.fd-initiative-row {
  border-bottom: 1px solid var(--fd-border-subtle);
  cursor: pointer;
}

.fd-initiative-row:hover {
  background: var(--fd-surface-hover);
}

.fd-initiative-row__name {
  display: flex;
  flex-direction: column;
  gap: 1px;
  min-width: 0;
}

.fd-initiative-row__name strong {
  overflow: hidden;
  font-size: var(--fd-text-12);
  font-weight: var(--fd-weight-medium);
  text-overflow: ellipsis;
  white-space: nowrap;
}

.fd-initiative-row__name small {
  overflow: hidden;
  color: var(--fd-text-muted);
  font-size: var(--fd-text-11);
  text-overflow: ellipsis;
  white-space: nowrap;
}

.fd-initiative-row__progress {
  min-width: 0;
}

.fd-initiative-row__muted {
  color: var(--fd-text-muted);
  font-size: var(--fd-text-11);
}

@media (max-width: 1199px) {
  .fd-initiative-grid {
    grid-template-columns: minmax(240px, 1.7fr) 110px minmax(160px, 1fr) 110px 80px;
  }

  .fd-initiative-grid > :nth-child(6) {
    display: none;
  }
}
</style>
