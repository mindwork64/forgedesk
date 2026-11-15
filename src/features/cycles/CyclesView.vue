<script setup lang="ts">
import { Plus, Timer } from '@lucide/vue'
import { computed, onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRoute, useRouter } from 'vue-router'

import IssueList from '@/components/issues/IssueList.vue'
import PageHeader from '@/components/layout/PageHeader.vue'
import PageToolbar from '@/components/layout/PageToolbar.vue'
import FdButton from '@/components/ui/FdButton.vue'
import FdEmptyState from '@/components/ui/FdEmptyState.vue'
import FdErrorState from '@/components/ui/FdErrorState.vue'
import FdLoadingSkeleton from '@/components/ui/FdLoadingSkeleton.vue'
import FdSelect from '@/components/ui/FdSelect.vue'
import { api } from '@/lib/ipc'
import { cycleRange, cycleStatusLabel } from '@/lib/planning'
import { useAppStore } from '@/stores/app'
import { usePlanningStore } from '@/stores/planning'
import type { Cycle, IssueSummary } from '@/types/ipc'

import CycleDialog from './CycleDialog.vue'

const planning = usePlanningStore()
const app = useAppStore()
const route = useRoute()
const router = useRouter()
const { t } = useI18n()

const selectedId = ref('')
const dialogOpen = ref(false)
const editing = ref<Cycle | null>(null)
const allIssues = ref<IssueSummary[]>([])
const addIssueId = ref('')
const removeIssueId = ref('')

const selectedCycle = computed(
  () => planning.cycles.find((cycle) => cycle.id === selectedId.value) ?? null,
)
const cycleIssues = computed(() => planning.cycle?.issues ?? [])
const availableIssues = computed(() => {
  const assigned = new Set(cycleIssues.value.map((issue) => issue.id))
  return allIssues.value.filter((issue) => !assigned.has(issue.id))
})

async function load(): Promise<void> {
  await Promise.all([planning.loadCycles(), loadAllIssues()])
  if (!selectedId.value) {
    const requested =
      typeof route.query.cycle === 'string' ? route.query.cycle : ''
    const active = planning.cycles.find((cycle) => cycle.id === requested)
    selectedId.value =
      active?.id ??
      planning.cycles.find((cycle) => cycle.status === 'active')?.id ??
      planning.cycles[0]?.id ??
      ''
  }
  if (selectedId.value) {
    await planning.loadCycle(selectedId.value)
  }
}

async function loadAllIssues(): Promise<void> {
  try {
    allIssues.value = await api.listIssues({
      sort: [{ field: 'updatedAt', direction: 'desc' }],
      includeArchived: false,
      limit: 500,
    })
  } catch {
    allIssues.value = []
  }
}

async function selectCycle(id: string): Promise<void> {
  selectedId.value = id
  addIssueId.value = ''
  removeIssueId.value = ''
  if (!id) {
    return
  }
  await router.replace({ path: '/cycles', query: { cycle: id } })
  await planning.loadCycle(id)
}

async function addIssue(issueId: string): Promise<void> {
  if (!issueId || !selectedId.value) {
    return
  }
  const ok = await planning.assignCycleIssue(selectedId.value, issueId, true)
  addIssueId.value = ''
  if (ok) {
    await planning.loadCycles()
  }
}

async function removeIssue(issueId: string): Promise<void> {
  if (!issueId || !selectedId.value) {
    return
  }
  const ok = await planning.assignCycleIssue(selectedId.value, issueId, false)
  removeIssueId.value = ''
  if (ok) {
    await planning.loadCycles()
  }
}

function openIssue(issueId: string): void {
  void router.push({ path: '/my-issues', query: { issue: issueId } })
}

function openCreate(): void {
  editing.value = null
  dialogOpen.value = true
}

function openEdit(): void {
  editing.value = selectedCycle.value
  dialogOpen.value = true
}

async function onSaved(cycle: Cycle): Promise<void> {
  await planning.loadCycles()
  await selectCycle(cycle.id)
}

watch(
  () => route.query.cycle,
  (id) => {
    if (typeof id === 'string' && id !== selectedId.value) {
      selectedId.value = id
      void planning.loadCycle(id)
    }
  },
)

onMounted(load)
</script>

<template>
  <div class="fd-cycles-page">
    <PageHeader
      :title="t('pages.cycles.title')"
      :description="t('pages.cycles.description')"
    >
      <template #actions>
        <FdButton v-if="selectedCycle" variant="ghost" @click="openEdit">
          {{ t('cycles.editCycle') }}
        </FdButton>
        <FdButton variant="primary" @click="openCreate">
          <Plus :size="14" aria-hidden="true" />
          {{ t('cycles.newCycle') }}
        </FdButton>
      </template>
    </PageHeader>

    <PageToolbar v-if="planning.cycles.length > 0">
      <template #left>
        <FdSelect
          :model-value="selectedId"
          class="fd-cycle-select"
          :aria-label="t('cycles.selectCycle')"
          @update:model-value="selectCycle"
        >
          <option v-for="cycle in planning.cycles" :key="cycle.id" :value="cycle.id">
            {{ cycle.name }}
          </option>
        </FdSelect>
        <span v-if="selectedCycle" class="fd-cycle-range">
          {{ cycleRange(selectedCycle, app.resolvedLocale) }}
        </span>
      </template>
      <template #right>
        <FdSelect
          :model-value="addIssueId"
          class="fd-cycle-issue-select"
          :aria-label="t('cycles.addIssue')"
          :disabled="!selectedCycle"
          @update:model-value="addIssue"
        >
          <option value="">{{ t('cycles.addIssue') }}</option>
          <option v-for="issue in availableIssues" :key="issue.id" :value="issue.id">
            {{ issue.key }} · {{ issue.title }}
          </option>
        </FdSelect>
        <FdSelect
          v-if="cycleIssues.length > 0"
          :model-value="removeIssueId"
          class="fd-cycle-issue-select"
          :aria-label="t('common.remove')"
          @update:model-value="removeIssue"
        >
          <option value="">{{ t('common.remove') }}</option>
          <option v-for="issue in cycleIssues" :key="issue.id" :value="issue.id">
            {{ issue.key }} · {{ issue.title }}
          </option>
        </FdSelect>
      </template>
    </PageToolbar>

    <div class="fd-cycles-content">
      <div v-if="planning.loading && planning.cycles.length === 0" class="fd-cycles-state">
        <FdLoadingSkeleton :rows="8" :height="42" />
      </div>
      <div v-else-if="planning.error" class="fd-cycles-state">
        <FdErrorState
          :title="t('cycles.createError')"
          :message="planning.error.message"
          :code="planning.error.code"
          :detail="planning.error.detail"
          :retry-label="t('common.retry')"
          @retry="load"
        />
      </div>
      <div v-else-if="planning.cycles.length === 0" class="fd-cycles-state">
        <FdEmptyState
          :icon="Timer"
          :title="t('cycles.noCyclesTitle')"
          :body="t('cycles.noCyclesBody')"
        >
          <FdButton variant="primary" @click="openCreate">
            <Plus :size="14" aria-hidden="true" />
            {{ t('cycles.createFirst') }}
          </FdButton>
        </FdEmptyState>
      </div>
      <template v-else-if="selectedCycle && planning.cycle">
        <div class="fd-cycle-summary">
          <div class="fd-cycle-summary__head">
            <div>
              <h2>{{ selectedCycle.name }}</h2>
              <p>
                {{ cycleStatusLabel(selectedCycle.status, t) }} ·
                {{ cycleRange(selectedCycle, app.resolvedLocale) }}
              </p>
            </div>
            <span class="fd-cycle-summary__days">
              {{ t('cycles.daysRemaining') }}: {{ selectedCycle.summary.daysRemaining }}
            </span>
          </div>
          <div class="fd-cycle-metrics">
            <div class="fd-cycle-metric">
              <span>{{ t('cycles.issueCount') }}</span>
              <strong>{{ selectedCycle.summary.issueCount }}</strong>
            </div>
            <div class="fd-cycle-metric">
              <span>{{ t('cycles.completedCount') }}</span>
              <strong>{{ selectedCycle.summary.completedCount }}</strong>
            </div>
            <div class="fd-cycle-metric">
              <span>{{ t('cycles.totalEstimate') }}</span>
              <strong>{{ selectedCycle.summary.totalEstimate }}</strong>
            </div>
            <div class="fd-cycle-metric">
              <span>{{ t('cycles.completedEstimate') }}</span>
              <strong>{{ selectedCycle.summary.completedEstimate }}</strong>
            </div>
            <div class="fd-cycle-metric">
              <span>{{ t('cycles.carryOver') }}</span>
              <strong>{{ selectedCycle.summary.carryOverCount }}</strong>
            </div>
          </div>
        </div>

        <div class="fd-cycle-issues">
          <div class="fd-cycle-issues__head">
            <h2>{{ t('cycles.issues') }}</h2>
            <span>{{ t('planning.issueCount', { count: cycleIssues.length }) }}</span>
          </div>
          <FdEmptyState
            v-if="cycleIssues.length === 0"
            :icon="Timer"
            :title="t('cycles.noIssues')"
            :body="t('cycles.addIssue')"
          />
          <IssueList
            v-else
            :issues="cycleIssues"
            :selected-ids="[]"
            @select="openIssue"
          />
        </div>

        <section class="fd-cycle-analytics">
          <h2>{{ t('cycles.analytics') }}</h2>
          <p>{{ t('cycles.analyticsEmpty') }}</p>
        </section>
      </template>
    </div>

    <CycleDialog v-model:open="dialogOpen" :cycle="editing" @saved="onSaved" />
  </div>
</template>

<style scoped>
.fd-cycles-page {
  display: flex;
  flex-direction: column;
  width: 100%;
  height: 100%;
  min-width: 0;
  min-height: 0;
  overflow: hidden;
}

.fd-cycle-select {
  width: 220px;
}

.fd-cycle-range {
  color: var(--fd-text-muted);
  font-size: var(--fd-text-11);
  white-space: nowrap;
}

.fd-cycle-issue-select {
  width: 230px;
}

.fd-cycles-content {
  display: flex;
  flex: 1;
  flex-direction: column;
  min-height: 0;
  overflow: auto;
}

.fd-cycles-state {
  display: flex;
  justify-content: center;
  padding: var(--fd-space-6) var(--fd-content-pad);
}

.fd-cycle-summary {
  flex: none;
  padding: var(--fd-space-4) var(--fd-content-pad);
  border-bottom: 1px solid var(--fd-border);
}

.fd-cycle-summary__head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: var(--fd-space-3);
}

.fd-cycle-summary__head h2 {
  margin: 0;
  font-size: var(--fd-text-16);
  font-weight: var(--fd-weight-semibold);
}

.fd-cycle-summary__head p {
  margin: 3px 0 0;
  color: var(--fd-text-muted);
  font-size: var(--fd-text-11);
}

.fd-cycle-summary__days {
  color: var(--fd-text-secondary);
  font-size: var(--fd-text-12);
}

.fd-cycle-metrics {
  display: grid;
  grid-template-columns: repeat(5, minmax(110px, 1fr));
  gap: var(--fd-space-3);
  margin-top: var(--fd-space-3);
}

.fd-cycle-metric {
  display: flex;
  flex-direction: column;
  gap: 2px;
  padding: 10px 12px;
  border: 1px solid var(--fd-border-subtle);
  border-radius: var(--fd-radius-control);
  background: var(--fd-surface);
}

.fd-cycle-metric span {
  color: var(--fd-text-muted);
  font-size: var(--fd-text-11);
}

.fd-cycle-metric strong {
  font-size: var(--fd-text-18);
  font-variant-numeric: tabular-nums;
}

.fd-cycle-issues {
  display: flex;
  flex: 1;
  flex-direction: column;
  min-height: 260px;
}

.fd-cycle-issues__head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  flex: none;
  height: 36px;
  padding: 0 var(--fd-content-pad);
  border-bottom: 1px solid var(--fd-border-subtle);
}

.fd-cycle-issues__head h2 {
  margin: 0;
  font-size: var(--fd-text-12);
  font-weight: var(--fd-weight-semibold);
}

.fd-cycle-issues__head span {
  color: var(--fd-text-muted);
  font-size: var(--fd-text-11);
}

.fd-cycle-analytics {
  flex: none;
  padding: var(--fd-space-4) var(--fd-content-pad);
  border-top: 1px solid var(--fd-border);
}

.fd-cycle-analytics h2 {
  margin: 0;
  font-size: var(--fd-text-12);
  font-weight: var(--fd-weight-semibold);
}

.fd-cycle-analytics p {
  margin: 4px 0 0;
  color: var(--fd-text-muted);
  font-size: var(--fd-text-11);
}

@media (max-width: 1199px) {
  .fd-cycle-metrics {
    grid-template-columns: repeat(3, minmax(110px, 1fr));
  }

  .fd-cycle-issue-select {
    width: 190px;
  }
}

@media (max-width: 899px) {
  .fd-cycle-metrics {
    grid-template-columns: repeat(2, minmax(110px, 1fr));
  }
}
</style>
