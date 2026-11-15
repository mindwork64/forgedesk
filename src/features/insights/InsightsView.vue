<script setup lang="ts">
import { BarChart3 } from '@lucide/vue'
import { computed, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'

import PageHeader from '@/components/layout/PageHeader.vue'
import PageToolbar from '@/components/layout/PageToolbar.vue'
import FdButton from '@/components/ui/FdButton.vue'
import FdEmptyState from '@/components/ui/FdEmptyState.vue'
import FdErrorState from '@/components/ui/FdErrorState.vue'
import FdLoadingSkeleton from '@/components/ui/FdLoadingSkeleton.vue'
import FdSegmented from '@/components/ui/FdSegmented.vue'
import { useAppStore } from '@/stores/app'
import { useInsightsStore } from '@/stores/insights'

const insights = useInsightsStore()
const app = useAppStore()
const { t } = useI18n()

const ranges = computed(() => [
  { value: '30', label: t('insightsPage.days30') },
  { value: '90', label: t('insightsPage.days90') },
  { value: '365', label: t('insightsPage.days365') },
])
const summary = computed(() => insights.summary)
const hasData = computed(
  () =>
    (summary.value?.totalCreated ?? 0) > 0 ||
    (summary.value?.totalCompleted ?? 0) > 0 ||
    (summary.value?.openCount ?? 0) > 0,
)
const maxThroughput = computed(() =>
  Math.max(1, ...(summary.value?.throughput.map((point) => point.count) ?? [1])),
)
const linePoints = computed(() => {
  const points = summary.value?.throughput ?? []
  if (points.length === 0) return ''
  return points
    .map((point, index) => {
      const x = 20 + (index / Math.max(1, points.length - 1)) * 760
      const y = 180 - (point.count / maxThroughput.value) * 140
      return `${x},${y}`
    })
    .join(' ')
})
const maxStatus = computed(() =>
  Math.max(1, ...(summary.value?.byStatus.map((bucket) => bucket.count) ?? [1])),
)
const maxProject = computed(() =>
  Math.max(1, ...(summary.value?.byProject.map((bucket) => bucket.count) ?? [1])),
)

function dataThrough(): string {
  return summary.value
    ? new Date(summary.value.dataThrough).toLocaleString(app.resolvedLocale)
    : '—'
}

function avgCycle(): string {
  if (summary.value?.avgCycleDays == null) return '—'
  return `${summary.value.avgCycleDays.toFixed(1)} ${t('insightsPage.days')}`
}

onMounted(() => {
  void insights.load()
})
</script>

<template>
  <div class="fd-insights-page">
    <PageHeader :title="t('pages.insights.title')" :description="t('pages.insights.description')">
      <template #actions>
        <span class="fd-insights-through">{{ t('insightsPage.dataThrough') }}: {{ dataThrough() }}</span>
      </template>
    </PageHeader>

    <PageToolbar>
      <template #left>
        <FdSegmented
          :model-value="String(insights.rangeDays)"
          :options="ranges"
          :label="t('insightsPage.range')"
          @update:model-value="insights.setRange(Number($event))"
        />
      </template>
      <template #right>
        <FdButton variant="ghost" size="sm" @click="insights.load()">
          {{ t('insightsPage.refresh') }}
        </FdButton>
      </template>
    </PageToolbar>

    <div class="fd-insights-content">
      <div v-if="insights.loading" class="fd-insights-state">
        <FdLoadingSkeleton :rows="6" :height="56" />
      </div>
      <div v-else-if="insights.error" class="fd-insights-state">
        <FdErrorState
          :title="t('insightsPage.loadError')"
          :message="insights.error.message"
          :code="insights.error.code"
          :detail="insights.error.detail"
          @retry="insights.load()"
        />
      </div>
      <FdEmptyState
        v-else-if="!hasData"
        :icon="BarChart3"
        :title="t('insightsPage.empty')"
        :body="t('insightsPage.emptyBody')"
      />
      <template v-else-if="summary">
        <section class="fd-insight-metrics">
          <div class="fd-insight-card">
            <span>{{ t('insightsPage.created') }}</span>
            <strong>{{ summary.totalCreated }}</strong>
          </div>
          <div class="fd-insight-card">
            <span>{{ t('insightsPage.completed') }}</span>
            <strong>{{ summary.totalCompleted }}</strong>
          </div>
          <div class="fd-insight-card">
            <span>{{ t('insightsPage.open') }}</span>
            <strong>{{ summary.openCount }}</strong>
          </div>
          <div class="fd-insight-card">
            <span>{{ t('insightsPage.overdue') }}</span>
            <strong>{{ summary.overdueCount }}</strong>
          </div>
          <div class="fd-insight-card">
            <span>{{ t('insightsPage.avgCycle') }}</span>
            <strong>{{ avgCycle() }}</strong>
          </div>
        </section>

        <section class="fd-insight-panel">
          <h2>{{ t('insightsPage.throughput') }}</h2>
          <svg class="fd-insight-chart" viewBox="0 0 800 210" role="img" :aria-label="t('insightsPage.throughput')">
            <line x1="20" y1="180" x2="780" y2="180" class="fd-insight-axis" />
            <polyline :points="linePoints" class="fd-insight-line" />
            <circle
              v-for="(point, index) in summary.throughput"
              :key="point.label"
              :cx="20 + (index / Math.max(1, summary.throughput.length - 1)) * 760"
              :cy="180 - (point.count / maxThroughput) * 140"
              r="3"
              class="fd-insight-dot"
            />
          </svg>
          <div class="fd-insight-labels">
            <span v-for="point in summary.throughput" :key="point.label">{{ point.label }}</span>
          </div>
        </section>

        <div class="fd-insight-split">
          <section class="fd-insight-panel">
            <h2>{{ t('insightsPage.byStatus') }}</h2>
            <div v-for="bucket in summary.byStatus" :key="bucket.label" class="fd-insight-bar-row">
              <span>{{ bucket.label }}</span>
              <div class="fd-insight-track"><span :style="{ width: `${(bucket.count / maxStatus) * 100}%` }" /></div>
              <strong>{{ bucket.count }}</strong>
            </div>
          </section>
          <section class="fd-insight-panel">
            <h2>{{ t('insightsPage.byProject') }}</h2>
            <div v-for="bucket in summary.byProject" :key="bucket.label" class="fd-insight-bar-row">
              <span>{{ bucket.label }}</span>
              <div class="fd-insight-track"><span :style="{ width: `${(bucket.count / maxProject) * 100}%` }" /></div>
              <strong>{{ bucket.count }}</strong>
            </div>
          </section>
        </div>
      </template>
    </div>
  </div>
</template>

<style scoped>
.fd-insights-page { display: flex; flex-direction: column; width: 100%; height: 100%; min-width: 0; min-height: 0; overflow: hidden; }
.fd-insights-through { color: var(--fd-text-muted); font-size: var(--fd-text-11); }
.fd-insights-content { flex: 1; min-height: 0; overflow: auto; padding: var(--fd-space-4) var(--fd-content-pad) var(--fd-space-6); }
.fd-insights-state { padding: var(--fd-space-4) 0; }
.fd-insight-metrics { display: grid; grid-template-columns: repeat(5, minmax(0, 1fr)); gap: var(--fd-space-2); margin-bottom: var(--fd-space-4); }
.fd-insight-card { display: flex; flex-direction: column; gap: 4px; padding: 10px 12px; border: 1px solid var(--fd-border-subtle); border-radius: var(--fd-radius-control); background: var(--fd-surface); }
.fd-insight-card span { color: var(--fd-text-muted); font-size: var(--fd-text-11); }
.fd-insight-card strong { font-size: 22px; font-weight: var(--fd-weight-semibold); }
.fd-insight-panel { padding: 10px 12px; border: 1px solid var(--fd-border-subtle); border-radius: var(--fd-radius-control); background: var(--fd-surface); }
.fd-insight-panel h2 { margin: 0 0 8px; color: var(--fd-text-muted); font-size: var(--fd-text-11); font-weight: var(--fd-weight-medium); text-transform: uppercase; }
.fd-insight-chart { width: 100%; height: 210px; }
.fd-insight-axis { stroke: var(--fd-border); stroke-width: 1; }
.fd-insight-line { fill: none; stroke: var(--fd-accent); stroke-width: 2; }
.fd-insight-dot { fill: var(--fd-accent); }
.fd-insight-labels { display: grid; grid-template-columns: repeat(12, 1fr); color: var(--fd-text-muted); font-size: 9px; text-align: center; }
.fd-insight-split { display: grid; grid-template-columns: 1fr 1fr; gap: var(--fd-space-3); margin-top: var(--fd-space-3); }
.fd-insight-bar-row { display: grid; grid-template-columns: 110px minmax(0, 1fr) 32px; gap: 8px; align-items: center; min-height: 26px; color: var(--fd-text-secondary); font-size: var(--fd-text-11); }
.fd-insight-track { height: 7px; overflow: hidden; border-radius: 4px; background: var(--fd-bg); }
.fd-insight-track span { display: block; height: 100%; border-radius: inherit; background: var(--fd-accent); }
.fd-insight-bar-row strong { text-align: right; font-weight: var(--fd-weight-medium); }
@media (max-width: 1100px) { .fd-insight-metrics { grid-template-columns: repeat(3, 1fr); } .fd-insight-split { grid-template-columns: 1fr; } }
</style>
