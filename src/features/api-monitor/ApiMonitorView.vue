<script setup lang="ts">
import { Gauge, Pause, Play } from '@lucide/vue'
import { computed, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'

import PageHeader from '@/components/layout/PageHeader.vue'
import FdButton from '@/components/ui/FdButton.vue'
import FdEmptyState from '@/components/ui/FdEmptyState.vue'
import FdErrorState from '@/components/ui/FdErrorState.vue'
import FdLoadingSkeleton from '@/components/ui/FdLoadingSkeleton.vue'
import { formatDateTime } from '@/lib/format'
import { useAppStore } from '@/stores/app'
import { useApiMonitorStore } from '@/stores/apiMonitor'

const api = useApiMonitorStore()
const app = useAppStore()
const { t } = useI18n()

const state = computed(() => api.state)
const graphql = computed(
  () => state.value?.budgets.find((budget) => budget.apiFamily === 'github_graphql') ?? null,
)
const rest = computed(
  () => state.value?.budgets.find((budget) => budget.apiFamily === 'github_rest') ?? null,
)
const resourceBudgets = computed(
  () =>
    state.value?.budgets.filter(
      (budget) =>
        budget.apiFamily !== 'github_graphql' && budget.apiFamily !== 'github_rest',
    ) ?? [],
)
const secondary = computed(
  () => state.value?.budgets.find((budget) => budget.secondaryPauseUntil) ?? null,
)
const inFlight = computed(
  () => state.value?.queue.filter((job) => job.state === 'running').length ?? 0,
)
const nextReset = computed(() => {
  const values = (state.value?.budgets ?? [])
    .map((budget) => budget.resetAtUtc)
    .filter((value): value is string => Boolean(value))
    .sort()
  return values[0] ?? null
})

function value(value: number | null | undefined): string {
  return value === null || value === undefined ? '—' : String(value)
}

function dateTime(value: string | null | undefined): string {
  return value ? formatDateTime(value, app.resolvedLocale) : '—'
}

function familyLabel(family: string): string {
  if (family === 'github_graphql') {
    return 'GraphQL'
  }
  if (family === 'github_rest') {
    return 'REST'
  }
  return family.replace('github_', '').replaceAll('_', ' ')
}

function budgetTitle(family: string): string {
  if (family === 'github_graphql') {
    return t('apiMonitor.graphql')
  }
  if (family === 'github_rest') {
    return t('apiMonitor.rest')
  }
  return family.replace('github_', '').replaceAll('_', ' ')
}

onMounted(() => {
  void api.load()
})
</script>

<template>
  <div class="fd-monitor-page">
    <PageHeader
      :title="t('pages.apiMonitor.title')"
      :description="t('pages.apiMonitor.description')"
    >
      <template #actions>
        <FdButton
          variant="ghost"
          size="md"
          @click="api.setPaused(!state?.control.paused)"
        >
          <Play v-if="state?.control.paused" :size="13" aria-hidden="true" />
          <Pause v-else :size="13" aria-hidden="true" />
          {{
            state?.control.paused
              ? t('apiMonitor.resume')
              : t('apiMonitor.pause')
          }}
        </FdButton>
        <FdButton variant="secondary" size="md" @click="api.exportDiagnostics()">
          {{ t('apiMonitor.exportDiagnostics') }}
        </FdButton>
      </template>
    </PageHeader>

    <div class="fd-monitor-content">
      <div v-if="api.loading" class="fd-monitor-state">
        <FdLoadingSkeleton :rows="8" :height="42" />
      </div>
      <div v-else-if="api.error" class="fd-monitor-state">
        <FdErrorState
          :title="t('apiMonitor.note')"
          :message="api.error.message"
          :code="api.error.code"
          :detail="api.error.detail"
          :retry-label="t('common.retry')"
          @retry="api.load()"
        />
      </div>
      <template v-else-if="state">
        <div class="fd-monitor-summary">
          <div class="fd-monitor-summary__item">
            <span>{{ t('apiMonitor.connection') }}</span>
            <strong>
              {{
                state.connected
                  ? t('apiMonitor.connectedAs', { login: state.accountLogin })
                  : t('apiMonitor.notConnected')
              }}
            </strong>
          </div>
          <div class="fd-monitor-summary__item">
            <span>{{ t('apiMonitor.scheduler') }}</span>
            <strong>
              {{
                state.control.paused
                  ? t('apiMonitor.paused')
                  : t('apiMonitor.idle')
              }}
            </strong>
          </div>
          <div class="fd-monitor-summary__item">
            <span>{{ t('apiMonitor.dispatcher') }}</span>
            <strong>{{ t('apiMonitor.inFlight', { count: inFlight }) }}</strong>
          </div>
          <div class="fd-monitor-summary__item">
            <span>{{ t('apiMonitor.lastObservation') }}</span>
            <strong>{{ dateTime(state.lastObservation) }}</strong>
          </div>
          <div class="fd-monitor-summary__item">
            <span>{{ t('apiMonitor.nextReset') }}</span>
            <strong>{{ dateTime(nextReset) }}</strong>
          </div>
        </div>

        <div class="fd-secondary-state" :class="{ 'fd-secondary-state--active': secondary }">
          {{ secondary ? t('apiMonitor.secondaryActive') : t('apiMonitor.secondaryClear') }}
        </div>

        <div v-if="state.budgets.length === 0" class="fd-monitor-empty">
          <FdEmptyState
            :icon="Gauge"
            :title="t('apiMonitor.noBudgets')"
            :body="t('apiMonitor.note')"
          />
        </div>
        <div v-else class="fd-budget-grid">
          <section
            v-for="budget in [graphql, rest].filter(Boolean)"
            :key="budget!.apiFamily"
            class="fd-budget-panel"
          >
            <div class="fd-budget-panel__head">
              <h2>{{ budgetTitle(budget!.apiFamily) }}</h2>
              <span v-if="budget!.isEstimate">{{ t('apiMonitor.estimate') }}</span>
            </div>
            <div class="fd-budget-values">
              <div><span>{{ t('apiMonitor.limit') }}</span><strong>{{ value(budget!.limitValue) }}</strong></div>
              <div><span>{{ t('apiMonitor.remaining') }}</span><strong>{{ value(budget!.remainingValue) }}</strong></div>
              <div><span>{{ t('apiMonitor.used') }}</span><strong>{{ value(budget!.usedValue) }}</strong></div>
            </div>
            <div class="fd-budget-meta">
              <span>{{ t('apiMonitor.githubRemaining') }}</span>
              <span>{{ t('apiMonitor.forYou') }}: {{ budget!.observedRequests }}</span>
              <span>{{ t('apiMonitor.backgroundAllowance') }}: {{ budget!.backgroundAllowance }}</span>
              <span>{{ t('apiMonitor.backgroundReserve') }}: {{ budget!.backgroundReserve }}</span>
              <span>{{ t('apiMonitor.observedAt') }}: {{ dateTime(budget!.observedAtUtc) }}</span>
              <span>{{ t('apiMonitor.source') }}: {{ budget!.source }}</span>
            </div>
          </section>

          <section v-if="resourceBudgets.length" class="fd-budget-panel fd-budget-panel--wide">
            <h2>{{ t('apiMonitor.resources') }}</h2>
            <div class="fd-resource-grid">
              <div v-for="budget in resourceBudgets" :key="budget.apiFamily + budget.resourceName">
                <span>{{ budgetTitle(budget.apiFamily) }} / {{ budget.resourceName }}</span>
                <strong>{{ value(budget.remainingValue) }} / {{ value(budget.limitValue) }}</strong>
              </div>
            </div>
          </section>
        </div>

        <p class="fd-monitor-note">{{ t('apiMonitor.note') }}</p>

        <section class="fd-monitor-section">
          <h2>{{ t('apiMonitor.queue') }}</h2>
          <p v-if="state.queue.length === 0" class="fd-monitor-muted">{{ t('apiMonitor.noQueue') }}</p>
          <div v-else class="fd-monitor-table" role="table">
            <div class="fd-monitor-row fd-monitor-row--head">
              <span>{{ t('apiMonitor.operation') }}</span>
              <span>{{ t('apiMonitor.priority') }}</span>
              <span>{{ t('apiMonitor.status') }}</span>
              <span>{{ t('apiMonitor.cost') }}</span>
              <span>{{ t('apiMonitor.retry') }}</span>
            </div>
            <div v-for="job in state.queue" :key="job.id" class="fd-monitor-row">
              <span>{{ job.operationName }}</span>
              <span>P{{ job.priority }}</span>
              <span>{{ job.state }}</span>
              <span>{{ job.estimatedCost }}</span>
              <span>{{ job.attempts }}</span>
            </div>
          </div>
        </section>

        <section class="fd-monitor-section">
          <h2>{{ t('apiMonitor.recentRequests') }}</h2>
          <p v-if="state.requests.length === 0" class="fd-monitor-muted">{{ t('apiMonitor.noRequests') }}</p>
          <div v-else class="fd-monitor-table fd-monitor-table--requests" role="table">
            <div class="fd-monitor-row fd-monitor-row--head">
              <span>{{ t('apiMonitor.time') }}</span>
              <span>{{ t('apiMonitor.operation') }}</span>
              <span>{{ t('apiMonitor.family') }}</span>
              <span>{{ t('apiMonitor.requestClass') }}</span>
              <span>{{ t('apiMonitor.cost') }}</span>
              <span>{{ t('apiMonitor.latency') }}</span>
              <span>{{ t('apiMonitor.status') }}</span>
            </div>
            <div v-for="request in state.requests" :key="request.id" class="fd-monitor-row">
              <span>{{ dateTime(request.receivedAtUtc) }}</span>
              <span>{{ request.operationName }}</span>
              <span>{{ familyLabel(request.apiFamily) }}</span>
              <span>{{ request.requestClass }}</span>
              <span>{{ request.actualCost }}</span>
              <span>{{ request.latencyMs ?? '—' }} ms</span>
              <span>{{ request.httpStatus ?? request.resultState }}</span>
            </div>
          </div>
        </section>

        <div class="fd-monitor-actions">
          <FdButton variant="ghost" size="sm" disabled>
            {{ t('apiMonitor.safeRead') }}
          </FdButton>
          <span class="fd-monitor-muted">{{ t('apiMonitor.retryUnavailable') }}</span>
        </div>
      </template>
    </div>
  </div>
</template>

<style scoped>
.fd-monitor-page {
  display: flex;
  flex-direction: column;
  width: 100%;
  height: 100%;
  min-width: 0;
  min-height: 0;
  overflow: hidden;
}

.fd-monitor-content {
  flex: 1;
  min-height: 0;
  padding: 0 var(--fd-content-pad) var(--fd-space-6);
  overflow: auto;
}

.fd-monitor-state {
  display: flex;
  justify-content: center;
  padding: var(--fd-space-6);
}

.fd-monitor-summary {
  display: grid;
  grid-template-columns: repeat(5, minmax(140px, 1fr));
  gap: var(--fd-space-2);
  margin-bottom: var(--fd-space-3);
}

.fd-monitor-summary__item {
  display: flex;
  flex-direction: column;
  gap: 2px;
  padding: 9px 11px;
  border: 1px solid var(--fd-border-subtle);
  border-radius: var(--fd-radius-control);
  background: var(--fd-surface);
}

.fd-monitor-summary__item span {
  color: var(--fd-text-muted);
  font-size: var(--fd-text-11);
}

.fd-monitor-summary__item strong {
  overflow: hidden;
  font-size: var(--fd-text-12);
  font-weight: var(--fd-weight-medium);
  text-overflow: ellipsis;
  white-space: nowrap;
}

.fd-secondary-state {
  padding: 8px 10px;
  margin-bottom: var(--fd-space-3);
  border: 1px solid var(--fd-success);
  border-radius: var(--fd-radius-control);
  color: var(--fd-success);
  font-size: var(--fd-text-11);
}

.fd-secondary-state--active {
  border-color: var(--fd-warning);
  color: var(--fd-warning);
}

.fd-monitor-empty {
  padding: var(--fd-space-6);
}

.fd-budget-grid {
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
  gap: var(--fd-space-3);
}

.fd-budget-panel {
  padding: var(--fd-space-3);
  border: 1px solid var(--fd-border);
  border-radius: var(--fd-radius-card);
  background: var(--fd-surface);
}

.fd-budget-panel--wide {
  grid-column: 1 / -1;
}

.fd-budget-panel__head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--fd-space-2);
}

.fd-budget-panel h2,
.fd-monitor-section h2 {
  margin: 0 0 var(--fd-space-2);
  font-size: var(--fd-text-12);
  font-weight: var(--fd-weight-semibold);
}

.fd-budget-panel__head span {
  color: var(--fd-warning);
  font-size: var(--fd-text-11);
}

.fd-budget-values {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: var(--fd-space-2);
}

.fd-budget-values div {
  display: flex;
  flex-direction: column;
  gap: 1px;
}

.fd-budget-values span,
.fd-budget-meta {
  color: var(--fd-text-muted);
  font-size: var(--fd-text-11);
}

.fd-budget-values strong {
  font-size: var(--fd-text-18);
  font-variant-numeric: tabular-nums;
}

.fd-budget-meta {
  display: flex;
  flex-wrap: wrap;
  gap: 4px 10px;
  margin-top: var(--fd-space-2);
  padding-top: var(--fd-space-2);
  border-top: 1px solid var(--fd-border-subtle);
}

.fd-resource-grid {
  display: grid;
  grid-template-columns: repeat(3, minmax(160px, 1fr));
  gap: var(--fd-space-2);
}

.fd-resource-grid div {
  display: flex;
  flex-direction: column;
  gap: 2px;
  padding: 7px 9px;
  border: 1px solid var(--fd-border-subtle);
  border-radius: var(--fd-radius-control);
  background: var(--fd-bg);
}

.fd-resource-grid span {
  color: var(--fd-text-muted);
  font-size: var(--fd-text-11);
}

.fd-resource-grid strong {
  font-size: var(--fd-text-12);
  font-variant-numeric: tabular-nums;
}

.fd-monitor-note {
  margin: var(--fd-space-2) 0 var(--fd-space-4);
  color: var(--fd-text-muted);
  font-size: var(--fd-text-11);
}

.fd-monitor-section {
  margin-top: var(--fd-space-4);
}

.fd-monitor-muted {
  margin: 0;
  color: var(--fd-text-muted);
  font-size: var(--fd-text-11);
}

.fd-monitor-table {
  border-top: 1px solid var(--fd-border);
}

.fd-monitor-row {
  display: grid;
  grid-template-columns: minmax(160px, 1.4fr) 70px 100px 80px 60px;
  gap: var(--fd-space-2);
  align-items: center;
  min-height: 30px;
  border-bottom: 1px solid var(--fd-border-subtle);
  color: var(--fd-text-secondary);
  font-size: var(--fd-text-11);
}

.fd-monitor-table--requests .fd-monitor-row {
  grid-template-columns: 160px minmax(160px, 1.4fr) 110px 120px 70px 80px 70px;
}

.fd-monitor-row--head {
  min-height: 28px;
  color: var(--fd-text-muted);
}

.fd-monitor-actions {
  display: flex;
  align-items: center;
  gap: var(--fd-space-2);
  margin-top: var(--fd-space-4);
}

@media (max-width: 1199px) {
  .fd-monitor-summary {
    grid-template-columns: repeat(3, minmax(140px, 1fr));
  }

  .fd-budget-grid {
    grid-template-columns: 1fr;
  }

  .fd-budget-panel--wide {
    grid-column: auto;
  }
}
</style>
