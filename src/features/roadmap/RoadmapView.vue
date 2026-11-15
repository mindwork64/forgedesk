<script setup lang="ts">
import { Map } from '@lucide/vue'
import { computed, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'

import PageHeader from '@/components/layout/PageHeader.vue'
import PageToolbar from '@/components/layout/PageToolbar.vue'
import FdEmptyState from '@/components/ui/FdEmptyState.vue'
import FdErrorState from '@/components/ui/FdErrorState.vue'
import FdLoadingSkeleton from '@/components/ui/FdLoadingSkeleton.vue'
import FdSegmented from '@/components/ui/FdSegmented.vue'
import FdSelect from '@/components/ui/FdSelect.vue'
import { formatShortDate, statusColor } from '@/lib/planning'
import { useAppStore } from '@/stores/app'
import { usePlanningStore } from '@/stores/planning'
import type { PlanningStatus } from '@/types/ipc'

const planning = usePlanningStore()
const app = useAppStore()
const { t } = useI18n()

const level = ref<'projects' | 'initiatives'>('projects')
const zoom = ref('3')

const levelOptions = computed(() => [
  { value: 'projects', label: t('roadmap.projects') },
  { value: 'initiatives', label: t('roadmap.initiatives') },
])
const zoomOptions = [
  { value: '1', label: t('roadmap.months', { count: 1 }) },
  { value: '3', label: t('roadmap.months', { count: 3 }) },
  { value: '6', label: t('roadmap.months', { count: 6 }) },
  { value: '12', label: t('roadmap.months', { count: 12 }) },
]

interface Row {
  id: string
  name: string
  start: string
  end: string
  status: PlanningStatus
  percent: number
}

const rows = computed<Row[]>(() => {
  if (level.value === 'projects') {
    return planning.projects
      .filter((project) => project.startDate || project.targetDate)
      .map((project) => ({
        id: project.id,
        name: project.name,
        start: project.startDate ?? project.targetDate!,
        end: project.targetDate ?? project.startDate!,
        status: project.status,
        percent: project.progress.percent,
      }))
  }
  return planning.initiatives
    .filter((initiative) => initiative.startDate || initiative.targetDate)
    .map((initiative) => ({
      id: initiative.id,
      name: initiative.name,
      start: initiative.startDate ?? initiative.targetDate!,
      end: initiative.targetDate ?? initiative.startDate!,
      status: initiative.status,
      percent: initiative.progress.percent,
    }))
})

const range = computed(() => {
  const today = new Date()
  const months = Number(zoom.value)
  const values = rows.value.flatMap((row) => [
    new Date(`${row.start}T00:00:00`),
    new Date(`${row.end}T00:00:00`),
  ])
  const min = values.length
    ? new Date(Math.min(...values.map((date) => date.getTime())))
    : new Date(today.getTime() - 14 * 86_400_000)
  const max = values.length
    ? new Date(Math.max(...values.map((date) => date.getTime())))
    : new Date(today.getTime() + 45 * 86_400_000)
  const center = new Date((min.getTime() + max.getTime()) / 2)
  const start = new Date(center)
  start.setMonth(start.getMonth() - Math.floor(months / 2))
  const end = new Date(center)
  end.setMonth(end.getMonth() + Math.ceil(months / 2))
  return { start, end }
})

const totalMs = computed(
  () => Math.max(1, range.value.end.getTime() - range.value.start.getTime()),
)
const todayPercent = computed(() => {
  const now = Date.now()
  return ((now - range.value.start.getTime()) / totalMs.value) * 100
})

function barLeft(row: Row): number {
  const start = new Date(`${row.start}T00:00:00`).getTime()
  return Math.max(
    0,
    ((start - range.value.start.getTime()) / totalMs.value) * 100,
  )
}

function barWidth(row: Row): number {
  const start = new Date(`${row.start}T00:00:00`).getTime()
  const end = new Date(`${row.end}T00:00:00`).getTime()
  const left = Math.max(
    0,
    ((start - range.value.start.getTime()) / totalMs.value) * 100,
  )
  const right = Math.min(
    100,
    ((end - range.value.start.getTime()) / totalMs.value) * 100,
  )
  return Math.max(1, right - left)
}

function label(value: string): string {
  return formatShortDate(value, app.resolvedLocale)
}

onMounted(async () => {
  await Promise.all([planning.loadProjects(), planning.loadInitiatives()])
})
</script>

<template>
  <div class="fd-roadmap-page">
    <PageHeader
      :title="t('pages.roadmap.title')"
      :description="t('pages.roadmap.description')"
    />

    <PageToolbar>
      <template #left>
        <FdSegmented
          :model-value="level"
          :options="levelOptions"
          :label="t('roadmap.level')"
          @update:model-value="level = $event as 'projects' | 'initiatives'"
        />
      </template>
      <template #right>
        <span class="fd-roadmap-zoom-label">{{ t('roadmap.zoom') }}</span>
        <FdSelect v-model="zoom" class="fd-roadmap-zoom" :aria-label="t('roadmap.zoom')">
          <option v-for="option in zoomOptions" :key="option.value" :value="option.value">
            {{ option.label }}
          </option>
        </FdSelect>
      </template>
    </PageToolbar>

    <div class="fd-roadmap-content">
      <div v-if="planning.loading" class="fd-roadmap-state">
        <FdLoadingSkeleton :rows="7" :height="42" />
      </div>
      <div v-else-if="planning.error" class="fd-roadmap-state">
        <FdErrorState
          :title="t('roadmap.noItems')"
          :message="planning.error.message"
          :code="planning.error.code"
          :detail="planning.error.detail"
          :retry-label="t('common.retry')"
          @retry="planning.loadProjects()"
        />
      </div>
      <div v-else-if="rows.length === 0" class="fd-roadmap-state">
        <FdEmptyState
          :icon="Map"
          :title="t('roadmap.noItems')"
          :body="t('roadmap.noDates')"
        />
      </div>
      <div v-else class="fd-roadmap">
        <div class="fd-roadmap__header">
          <span class="fd-roadmap__label-head">{{ t('roadmap.level') }}</span>
          <div class="fd-roadmap__axis">
            <span>{{ label(range.start.toISOString().slice(0, 10)) }}</span>
            <span>{{ t('roadmap.today') }}</span>
            <span>{{ label(range.end.toISOString().slice(0, 10)) }}</span>
          </div>
        </div>
        <div class="fd-roadmap__body">
          <div class="fd-roadmap__labels">
            <button
              v-for="row in rows"
              :key="row.id"
              type="button"
              class="fd-roadmap__label"
              @click="
                level === 'projects'
                  ? $router.push(`/projects/${row.id}`)
                  : $router.push(`/initiatives/${row.id}`)
              "
            >
              {{ row.name }}
            </button>
          </div>
          <div class="fd-roadmap__timeline">
            <div
              v-if="todayPercent >= 0 && todayPercent <= 100"
              class="fd-roadmap__today"
              :style="{ left: `${todayPercent}%` }"
            />
            <span
              v-for="tick in 6"
              :key="tick"
              class="fd-roadmap__gridline"
              :style="{ left: `${(tick - 1) * 20}%` }"
            />
            <div
              v-for="row in rows"
              :key="row.id"
              class="fd-roadmap__row"
            >
              <div
                class="fd-roadmap__bar"
                :style="{
                  left: `${barLeft(row)}%`,
                  width: `${barWidth(row)}%`,
                  background: statusColor(row.status),
                }"
                :title="`${row.name} · ${label(row.start)} – ${label(row.end)}`"
              >
                <span class="fd-roadmap__bar-progress" :style="{ width: `${row.percent}%` }" />
                <span class="fd-roadmap__bar-label">
                  {{ label(row.start) }} – {{ label(row.end) }}
                </span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.fd-roadmap-page {
  display: flex;
  flex-direction: column;
  width: 100%;
  height: 100%;
  min-width: 0;
  min-height: 0;
  overflow: hidden;
}

.fd-roadmap-zoom-label {
  color: var(--fd-text-muted);
  font-size: var(--fd-text-11);
}

.fd-roadmap-zoom {
  width: 130px;
}

.fd-roadmap-content {
  flex: 1;
  min-height: 0;
  overflow: auto;
}

.fd-roadmap-state {
  display: flex;
  justify-content: center;
  padding: var(--fd-space-6) var(--fd-content-pad);
}

.fd-roadmap {
  min-width: 900px;
  padding: var(--fd-space-3) var(--fd-content-pad) var(--fd-space-6);
}

.fd-roadmap__header {
  display: flex;
  min-height: 30px;
  border-bottom: 1px solid var(--fd-border);
  color: var(--fd-text-muted);
  font-size: var(--fd-text-11);
}

.fd-roadmap__label-head {
  flex: 0 0 220px;
  padding: 6px 8px 6px 0;
}

.fd-roadmap__axis {
  display: flex;
  flex: 1;
  justify-content: space-between;
  padding: 6px 0;
}

.fd-roadmap__body {
  display: flex;
}

.fd-roadmap__labels {
  flex: 0 0 220px;
  padding-top: 21px;
}

.fd-roadmap__label {
  display: flex;
  align-items: center;
  width: 100%;
  height: 38px;
  padding-right: 8px;
  border: 0;
  background: transparent;
  overflow: hidden;
  color: var(--fd-text-secondary);
  cursor: pointer;
  font-size: var(--fd-text-12);
  text-align: left;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.fd-roadmap__label:hover {
  color: var(--fd-accent);
}

.fd-roadmap__timeline {
  position: relative;
  flex: 1;
  padding-top: 21px;
}

.fd-roadmap__row {
  position: relative;
  height: 38px;
}

.fd-roadmap__gridline {
  position: absolute;
  top: 0;
  bottom: 0;
  width: 1px;
  background: var(--fd-border-subtle);
  pointer-events: none;
}

.fd-roadmap__today {
  position: absolute;
  top: 0;
  bottom: 0;
  width: 1px;
  background: var(--fd-accent);
  opacity: 0.55;
  pointer-events: none;
}

.fd-roadmap__bar {
  position: absolute;
  top: 9px;
  height: 20px;
  min-width: 6px;
  overflow: hidden;
  border-radius: 4px;
  opacity: 0.72;
}

.fd-roadmap__bar-progress {
  position: absolute;
  inset: 0 auto 0 0;
  background: rgba(255, 255, 255, 0.16);
}

.fd-roadmap__bar-label {
  position: relative;
  display: block;
  padding: 2px 6px;
  overflow: hidden;
  color: #fff;
  font-size: 10px;
  text-overflow: ellipsis;
  white-space: nowrap;
}
</style>
