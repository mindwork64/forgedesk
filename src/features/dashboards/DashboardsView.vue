<script setup lang="ts">
import { LayoutDashboard, Plus, Trash2 } from '@lucide/vue'
import { computed, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'

import PageHeader from '@/components/layout/PageHeader.vue'
import PageToolbar from '@/components/layout/PageToolbar.vue'
import FdButton from '@/components/ui/FdButton.vue'
import FdEmptyState from '@/components/ui/FdEmptyState.vue'
import FdErrorState from '@/components/ui/FdErrorState.vue'
import FdIconButton from '@/components/ui/FdIconButton.vue'
import FdLoadingSkeleton from '@/components/ui/FdLoadingSkeleton.vue'
import { api, toIpcError } from '@/lib/ipc'
import { useAppStore } from '@/stores/app'
import { useDashboardStore } from '@/stores/dashboard'
import type {
  ApiMonitorState,
  InsightsSummary,
  IssueSummary,
} from '@/types/ipc'

const dashboard = useDashboardStore()
const app = useAppStore()
const { t } = useI18n()
const edit = ref(false)
const insights = ref<InsightsSummary | null>(null)
const issues = ref<IssueSummary[]>([])
const monitor = ref<ApiMonitorState | null>(null)
const dataError = ref<string | null>(null)

const widgets = computed(() => dashboard.dashboard?.widgets ?? [])
const addable = [
  { kind: 'metric_open', width: 3 },
  { kind: 'metric_completed', width: 3 },
  { kind: 'metric_overdue', width: 3 },
  { kind: 'issues_recent', width: 6 },
  { kind: 'api_budget', width: 4 },
]

function title(kind: string): string {
  return t(`dashboardPage.widgets.${kind}`)
}

function metric(kind: string): number {
  if (!insights.value) return 0
  if (kind === 'metric_open') return insights.value.openCount
  if (kind === 'metric_completed') return insights.value.totalCompleted
  return insights.value.overdueCount
}

function dataThrough(): string {
  return insights.value
    ? new Date(insights.value.dataThrough).toLocaleString(app.resolvedLocale)
    : '—'
}

function apiSummary(): string {
  if (!monitor.value) return t('dashboardPage.noData')
  if (!monitor.value.connected) return t('dashboardPage.apiDisconnected')
  const budget = monitor.value.budgets[0]
  if (!budget) return t('dashboardPage.noObservations')
  return `${budget.remainingValue ?? '—'} / ${budget.limitValue ?? '—'}`
}

async function loadData(): Promise<void> {
  dataError.value = null
  try {
    const [nextInsights, nextIssues, nextMonitor] = await Promise.all([
      api.getInsights(30),
      api.listIssues({ limit: 5, sort: [{ field: 'updatedAt', direction: 'desc' }] }),
      api.getApiMonitor(),
    ])
    insights.value = nextInsights
    issues.value = nextIssues
    monitor.value = nextMonitor
  } catch (caught) {
    dataError.value = toIpcError(caught).message
  }
}

onMounted(async () => {
  await Promise.all([dashboard.load(), loadData()])
})
</script>

<template>
  <div class="fd-dash-page">
    <PageHeader :title="t('pages.dashboards.title')" :description="t('pages.dashboards.description')">
      <template #actions>
        <FdButton variant="ghost" @click="edit = !edit">
          {{ edit ? t('dashboardPage.done') : t('dashboardPage.edit') }}
        </FdButton>
      </template>
    </PageHeader>

    <PageToolbar v-if="edit">
      <template #left>
        <span class="fd-dash-hint">{{ t('dashboardPage.addWidget') }}</span>
        <FdButton
          v-for="item in addable"
          :key="item.kind"
          variant="ghost"
          size="sm"
          @click="dashboard.add(item.kind, item.width)"
        >
          <Plus :size="12" /> {{ title(item.kind) }}
        </FdButton>
      </template>
      <template #right>
        <span class="fd-dash-hint">{{ t('dashboardPage.dataThrough') }}: {{ dataThrough() }}</span>
      </template>
    </PageToolbar>

    <div class="fd-dash-content">
      <div v-if="dashboard.loading" class="fd-dash-state">
        <FdLoadingSkeleton :rows="4" :height="120" />
      </div>
      <div v-else-if="dashboard.error" class="fd-dash-state">
        <FdErrorState
          :title="t('dashboardPage.loadError')"
          :message="dashboard.error.message"
          :code="dashboard.error.code"
          :detail="dashboard.error.detail"
          @retry="dashboard.load()"
        />
      </div>
      <FdEmptyState
        v-else-if="widgets.length === 0"
        :icon="LayoutDashboard"
        :title="t('dashboardPage.empty')"
        :body="t('dashboardPage.emptyBody')"
      >
        <FdButton variant="primary" @click="dashboard.add('metric_open', 3)">
          {{ t('dashboardPage.widgets.metric_open') }}
        </FdButton>
      </FdEmptyState>
      <div v-else class="fd-dash-grid">
        <section
          v-for="widget in widgets"
          :key="widget.id"
          class="fd-dash-widget"
          :style="{ gridColumn: `span ${widget.width}` }"
        >
          <header>
            <h2>{{ title(widget.kind) }}</h2>
            <div v-if="edit" class="fd-dash-widget__edit">
              <select
                :value="widget.width"
                :aria-label="t('dashboardPage.width')"
                @change="dashboard.resize(widget.id, Number(($event.target as HTMLSelectElement).value))"
              >
                <option v-for="width in [3, 4, 6, 8, 12]" :key="width" :value="width">{{ width }}</option>
              </select>
              <FdIconButton :label="t('dashboardPage.moveUp')" @click="dashboard.move(widget.id, -1)">
                ↑
              </FdIconButton>
              <FdIconButton :label="t('dashboardPage.moveDown')" @click="dashboard.move(widget.id, 1)">
                ↓
              </FdIconButton>
              <FdIconButton :label="t('dashboardPage.remove')" @click="dashboard.remove(widget.id)">
                <Trash2 :size="14" />
              </FdIconButton>
            </div>
          </header>
          <template v-if="widget.kind.startsWith('metric_')">
            <strong class="fd-dash-metric">{{ metric(widget.kind) }}</strong>
            <span class="fd-dash-muted">{{ t('dashboardPage.dataThrough') }}: {{ dataThrough() }}</span>
          </template>
          <template v-else-if="widget.kind === 'issues_recent'">
            <p v-if="issues.length === 0" class="fd-dash-muted">{{ t('dashboardPage.noData') }}</p>
            <div v-for="issue in issues" :key="issue.id" class="fd-dash-issue">
              <span>{{ issue.key }}</span>
              <strong>{{ issue.title }}</strong>
              <span>{{ issue.status.name }}</span>
            </div>
          </template>
          <template v-else>
            <strong class="fd-dash-budget">{{ apiSummary() }}</strong>
            <span class="fd-dash-muted">{{ monitor?.control.paused ? t('dashboardPage.paused') : t('dashboardPage.active') }}</span>
          </template>
        </section>
      </div>
      <p v-if="dataError" class="fd-dash-error" role="alert">{{ dataError }}</p>
    </div>
  </div>
</template>

<style scoped>
.fd-dash-page { display: flex; flex-direction: column; width: 100%; height: 100%; min-width: 0; min-height: 0; overflow: hidden; }
.fd-dash-content { flex: 1; min-height: 0; overflow: auto; padding: var(--fd-space-4) var(--fd-content-pad) var(--fd-space-6); }
.fd-dash-state { padding: var(--fd-space-4) 0; }
.fd-dash-grid { display: grid; grid-template-columns: repeat(12, minmax(0, 1fr)); gap: var(--fd-space-3); align-items: start; }
.fd-dash-widget { display: flex; flex-direction: column; gap: 8px; min-width: 0; padding: 12px; border: 1px solid var(--fd-border-subtle); border-radius: var(--fd-radius-control); background: var(--fd-surface); }
.fd-dash-widget header { display: flex; align-items: center; justify-content: space-between; gap: 8px; }
.fd-dash-widget h2 { margin: 0; color: var(--fd-text-muted); font-size: var(--fd-text-11); font-weight: var(--fd-weight-medium); text-transform: uppercase; }
.fd-dash-widget__edit { display: flex; align-items: center; gap: 2px; }
.fd-dash-widget__edit select { height: 24px; border: 1px solid var(--fd-border); border-radius: 4px; background: var(--fd-bg); color: var(--fd-text-secondary); font-size: var(--fd-text-10); }
.fd-dash-metric { font-size: 34px; font-weight: var(--fd-weight-semibold); }
.fd-dash-budget { font-size: 22px; font-weight: var(--fd-weight-semibold); }
.fd-dash-muted { color: var(--fd-text-muted); font-size: var(--fd-text-11); }
.fd-dash-issue { display: grid; grid-template-columns: 62px minmax(0, 1fr) 90px; gap: 8px; align-items: center; min-height: 28px; border-bottom: 1px solid var(--fd-border-subtle); font-size: var(--fd-text-11); }
.fd-dash-issue span:first-child { color: var(--fd-text-muted); font-family: var(--fd-font-mono); font-size: var(--fd-text-10); }
.fd-dash-issue strong { overflow: hidden; font-weight: var(--fd-weight-medium); text-overflow: ellipsis; white-space: nowrap; }
.fd-dash-issue span:last-child { color: var(--fd-text-muted); text-align: right; }
.fd-dash-hint { color: var(--fd-text-muted); font-size: var(--fd-text-11); }
.fd-dash-error { color: var(--fd-danger); font-size: var(--fd-text-11); }
@media (max-width: 1000px) { .fd-dash-grid { grid-template-columns: repeat(6, minmax(0, 1fr)); } .fd-dash-widget { grid-column: span 6 !important; } }
</style>
