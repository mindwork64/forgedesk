<script setup lang="ts">
import { Activity as ActivityIcon, RefreshCw } from '@lucide/vue'
import { onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRouter } from 'vue-router'

import PageHeader from '@/components/layout/PageHeader.vue'
import FdButton from '@/components/ui/FdButton.vue'
import FdEmptyState from '@/components/ui/FdEmptyState.vue'
import FdErrorState from '@/components/ui/FdErrorState.vue'
import FdLoadingSkeleton from '@/components/ui/FdLoadingSkeleton.vue'
import { api, toIpcError } from '@/lib/ipc'
import { formatRelative } from '@/lib/format'
import { useAppStore } from '@/stores/app'
import type { ActivityFeedItem } from '@/types/ipc'

const { t, te } = useI18n()
const router = useRouter()
const app = useAppStore()
const events = ref<ActivityFeedItem[]>([])
const loading = ref(false)
const error = ref<string | null>(null)

async function load(): Promise<void> {
  loading.value = true
  error.value = null
  try {
    events.value = await api.activityList(150)
  } catch (caught) {
    error.value = toIpcError(caught).message
  } finally {
    loading.value = false
  }
}

function eventLabel(event: ActivityFeedItem): string {
  const key = `activityPage.events.${event.kind}`
  return te(key) ? t(key) : event.kind.replace(/_/g, ' ')
}

function eventDetails(event: ActivityFeedItem): string {
  if (!event.data || typeof event.data !== 'object' || Array.isArray(event.data)) {
    return t('activityPage.noDetails')
  }
  const data = event.data as Record<string, unknown>
  if (event.kind === 'status_changed' && data.from && data.to) {
    return t('activityPage.statusChange', {
      from: String(data.from),
      to: String(data.to),
    })
  }
  if (Array.isArray(data.fields)) return data.fields.map(String).join(', ')
  const entries = Object.entries(data)
  if (entries.length === 0) return t('activityPage.noDetails')
  return entries
    .map(([key, value]) => `${key}: ${Array.isArray(value) ? value.map(String).join(', ') : String(value)}`)
    .join(' · ')
}

function openIssue(event: ActivityFeedItem): void {
  void router.push({ path: '/my-issues', query: { issue: event.issueId } })
}

onMounted(() => {
  void load()
})
</script>

<template>
  <div class="fd-activity-page">
    <PageHeader :title="t('pages.activity.title')" :description="t('pages.activity.description')">
      <template #actions>
        <FdButton variant="secondary" :loading="loading" @click="load">
          <RefreshCw :size="14" aria-hidden="true" />
          {{ t('activityPage.refresh') }}
        </FdButton>
      </template>
    </PageHeader>

    <main class="fd-activity-scroll">
      <div v-if="loading && events.length === 0" class="fd-activity-state">
        <FdLoadingSkeleton :rows="7" :height="54" />
      </div>
      <div v-else-if="error" class="fd-activity-state">
        <FdErrorState
          :title="t('activityPage.loadError')"
          :message="error"
          :retry-label="t('common.retry')"
          @retry="load"
        />
      </div>
      <div v-else-if="events.length === 0" class="fd-activity-state">
        <FdEmptyState
          :icon="ActivityIcon"
          :title="t('activityPage.empty')"
          :body="t('activityPage.emptyBody')"
        />
      </div>
      <div v-else class="fd-activity-list">
        <button
          v-for="event in events"
          :key="event.id"
          class="fd-activity-row"
          type="button"
          @click="openIssue(event)"
        >
          <span class="fd-activity-marker" aria-hidden="true" />
          <span class="fd-activity-main">
            <span class="fd-activity-title">
              <strong>{{ eventLabel(event) }}</strong>
              <span>{{ event.issueKey }} · {{ event.issueTitle }}</span>
            </span>
            <small>{{ eventDetails(event) }}</small>
          </span>
          <span class="fd-activity-meta">
            <span>{{ event.source }}</span>
            <time :datetime="event.createdAt">{{ formatRelative(event.createdAt, app.resolvedLocale) }}</time>
          </span>
        </button>
      </div>
    </main>
  </div>
</template>

<style scoped>
.fd-activity-page {
  display: flex;
  flex-direction: column;
  width: 100%;
  height: 100%;
  min-width: 0;
  min-height: 0;
  overflow: hidden;
}
.fd-activity-scroll { flex: 1; min-height: 0; overflow: auto; }
.fd-activity-state { padding: var(--fd-space-6) var(--fd-content-pad); }
.fd-activity-list { padding: var(--fd-space-3) var(--fd-content-pad) var(--fd-space-8); }
.fd-activity-row {
  display: grid;
  grid-template-columns: 12px minmax(0, 1fr) auto;
  gap: var(--fd-space-3);
  align-items: center;
  width: 100%;
  min-height: 58px;
  padding: 9px 6px;
  border: 0;
  border-bottom: 1px solid var(--fd-border-subtle);
  background: transparent;
  color: var(--fd-text-primary);
  text-align: left;
  cursor: pointer;
}
.fd-activity-row:hover, .fd-activity-row:focus-visible {
  background: var(--fd-surface-hover);
  outline: none;
}
.fd-activity-marker {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  background: var(--fd-accent);
}
.fd-activity-main { display: flex; flex-direction: column; gap: 4px; min-width: 0; }
.fd-activity-title { display: flex; flex-wrap: wrap; gap: 8px; align-items: baseline; }
.fd-activity-title strong { font-size: var(--fd-text-12); font-weight: var(--fd-weight-medium); }
.fd-activity-title span { color: var(--fd-text-secondary); font-size: var(--fd-text-12); }
.fd-activity-main small { color: var(--fd-text-muted); font-size: var(--fd-text-11); overflow-wrap: anywhere; }
.fd-activity-meta { display: flex; flex-direction: column; gap: 3px; align-items: flex-end; color: var(--fd-text-muted); font-size: var(--fd-text-10); white-space: nowrap; }
.fd-activity-meta span { text-transform: uppercase; }
@media (max-width: 760px) {
  .fd-activity-row { grid-template-columns: 12px minmax(0, 1fr); }
  .fd-activity-meta { grid-column: 2; flex-direction: row; align-items: center; }
}
</style>
