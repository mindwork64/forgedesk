<script setup lang="ts">
import { Activity as ActivityIcon, RefreshCw } from '@lucide/vue'
import { computed, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRouter } from 'vue-router'

import PageHeader from '@/components/layout/PageHeader.vue'
import PageToolbar from '@/components/layout/PageToolbar.vue'
import FdButton from '@/components/ui/FdButton.vue'
import FdEmptyState from '@/components/ui/FdEmptyState.vue'
import FdErrorState from '@/components/ui/FdErrorState.vue'
import FdLoadingSkeleton from '@/components/ui/FdLoadingSkeleton.vue'
import FdSegmented from '@/components/ui/FdSegmented.vue'
import { formatRelative } from '@/lib/format'
import { api, toIpcError } from '@/lib/ipc'
import { useAppStore } from '@/stores/app'
import type { InboxEvent } from '@/types/ipc'

const app = useAppStore()
const router = useRouter()
const { t, te } = useI18n()
const events = ref<InboxEvent[]>([])
const filter = ref('all')
const loading = ref(false)
const error = ref<ReturnType<typeof toIpcError> | null>(null)

const filters = computed(() => [
  { value: 'all', label: t('activityPage.all') },
  { value: 'local', label: t('activityPage.local') },
  { value: 'github', label: t('activityPage.github') },
  { value: 'sync', label: t('activityPage.sync') },
])
const filteredEvents = computed(() =>
  filter.value === 'all' ? events.value : events.value.filter((event) => event.source === filter.value),
)
const groups = computed(() => {
  const today = new Date()
  today.setHours(0, 0, 0, 0)
  const yesterday = new Date(today)
  yesterday.setDate(today.getDate() - 1)
  const result = [
    { key: 'today', label: t('activityPage.today'), events: [] as InboxEvent[] },
    { key: 'yesterday', label: t('activityPage.yesterday'), events: [] as InboxEvent[] },
    { key: 'earlier', label: t('activityPage.earlier'), events: [] as InboxEvent[] },
  ]
  for (const event of filteredEvents.value) {
    const date = new Date(event.createdAt)
    if (date >= today) result[0]!.events.push(event)
    else if (date >= yesterday) result[1]!.events.push(event)
    else result[2]!.events.push(event)
  }
  return result.filter((group) => group.events.length > 0)
})

async function load(): Promise<void> {
  loading.value = true
  error.value = null
  try {
    events.value = await api.activityList()
  } catch (caught) {
    error.value = toIpcError(caught)
  } finally {
    loading.value = false
  }
}

function sourceLabel(source: InboxEvent['source']): string {
  return t(`activityPage.sources.${source}`)
}

function eventTitle(event: InboxEvent): string {
  const key = `activityPage.kinds.${event.kind}`
  return te(key) ? t(key) : event.title
}

function openEvent(event: InboxEvent): void {
  if (event.entityType === 'issue') {
    void router.push({ path: '/my-issues', query: { issue: event.entityId } })
  }
}

onMounted(() => {
  void load()
})
</script>

<template>
  <div class="fd-activity-page">
    <PageHeader :title="t('pages.activity.title')" :description="t('pages.activity.description')" />
    <PageToolbar>
      <template #left>
        <FdSegmented v-model="filter" :options="filters" :label="t('activityPage.filter')" />
      </template>
      <template #right>
        <span class="fd-activity-count">{{ t('activityPage.entryCount', { count: filteredEvents.length }) }}</span>
        <FdButton variant="secondary" size="sm" :loading="loading" @click="load">
          <RefreshCw :size="14" aria-hidden="true" />
          {{ t('activityPage.refresh') }}
        </FdButton>
      </template>
    </PageToolbar>

    <div class="fd-activity-scroll">
      <div v-if="loading && events.length === 0" class="fd-activity-state">
        <FdLoadingSkeleton :rows="7" :height="48" />
      </div>
      <div v-else-if="error" class="fd-activity-state">
        <FdErrorState
          :title="t('activityPage.loadError')"
          :message="error.message"
          :code="error.code"
          :detail="error.detail"
          @retry="load"
        />
      </div>
      <div v-else-if="filteredEvents.length === 0" class="fd-activity-state">
        <FdEmptyState
          :icon="ActivityIcon"
          :title="t('activityPage.emptyTitle')"
          :body="t('activityPage.emptyBody')"
        />
      </div>
      <div v-else class="fd-activity-list" data-testid="activity-list">
        <section v-for="group in groups" :key="group.key" class="fd-activity-group">
          <h2>{{ group.label }}</h2>
          <button
            v-for="event in group.events"
            :key="event.id"
            type="button"
            class="fd-activity-row"
            :data-testid="'activity-event'"
            @click="openEvent(event)"
          >
            <span class="fd-activity-marker" :class="`fd-activity-marker--${event.source}`" />
            <span class="fd-activity-main">
              <span class="fd-activity-title">
                <strong>{{ eventTitle(event) }}</strong>
                <small>{{ sourceLabel(event.source) }}</small>
                <small v-if="event.archived" class="fd-activity-archived">{{ t('activityPage.archivedInInbox') }}</small>
              </span>
              <span class="fd-activity-summary">{{ event.summary || event.entityTitle }}</span>
            </span>
            <span class="fd-activity-entity">{{ event.entityKey }}</span>
            <span class="fd-activity-time">{{ formatRelative(event.createdAt, app.resolvedLocale) }}</span>
          </button>
        </section>
      </div>
    </div>
  </div>
</template>

<style scoped>
.fd-activity-page { display: flex; flex-direction: column; width: 100%; height: 100%; min-width: 0; min-height: 0; overflow: hidden; }
.fd-activity-count { color: var(--fd-text-muted); font-size: var(--fd-text-11); font-variant-numeric: tabular-nums; }
.fd-activity-scroll { flex: 1; min-height: 0; overflow: auto; }
.fd-activity-state { display: flex; justify-content: center; padding: var(--fd-space-6) var(--fd-content-pad); }
.fd-activity-state :deep(.fd-skeleton) { max-width: 900px; }
.fd-activity-group h2 { position: sticky; top: 0; z-index: 1; margin: 0; padding: 8px var(--fd-content-pad) 4px; background: var(--fd-bg); color: var(--fd-text-muted); font-size: var(--fd-text-11); font-weight: var(--fd-weight-medium); text-transform: uppercase; }
.fd-activity-row { display: grid; grid-template-columns: 10px minmax(0, 1fr) 100px 100px; gap: var(--fd-space-2); align-items: center; width: 100%; min-height: 54px; padding: 6px var(--fd-content-pad); border: 0; border-bottom: 1px solid var(--fd-border-subtle); background: transparent; color: var(--fd-text); text-align: left; cursor: pointer; }
.fd-activity-row:hover, .fd-activity-row:focus-visible { background: var(--fd-surface-hover); outline: none; }
.fd-activity-marker { width: 6px; height: 6px; border-radius: 50%; background: var(--fd-text-muted); }
.fd-activity-marker--github { background: var(--fd-accent); }
.fd-activity-marker--sync { background: var(--fd-warning); }
.fd-activity-main { display: flex; flex-direction: column; gap: 2px; min-width: 0; }
.fd-activity-title { display: flex; align-items: baseline; gap: 8px; min-width: 0; }
.fd-activity-title strong { overflow: hidden; font-size: var(--fd-text-12); font-weight: var(--fd-weight-medium); text-overflow: ellipsis; white-space: nowrap; }
.fd-activity-title small { flex: none; color: var(--fd-text-muted); font-size: var(--fd-text-10); text-transform: uppercase; }
.fd-activity-title .fd-activity-archived { color: var(--fd-warning); }
.fd-activity-summary { overflow: hidden; color: var(--fd-text-muted); font-size: var(--fd-text-11); text-overflow: ellipsis; white-space: nowrap; }
.fd-activity-entity, .fd-activity-time { color: var(--fd-text-muted); font-size: var(--fd-text-10); white-space: nowrap; }
@media (max-width: 900px) { .fd-activity-row { grid-template-columns: 10px minmax(0, 1fr) 80px; } .fd-activity-time { display: none; } }
</style>
