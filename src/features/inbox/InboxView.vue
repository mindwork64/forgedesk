<script setup lang="ts">
import { Archive, Inbox as InboxIcon, Mail, MailOpen, X } from '@lucide/vue'
import { computed, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRouter } from 'vue-router'

import EntityDetailPane from '@/components/issues/EntityDetailPane.vue'
import PageHeader from '@/components/layout/PageHeader.vue'
import PageToolbar from '@/components/layout/PageToolbar.vue'
import FdButton from '@/components/ui/FdButton.vue'
import FdEmptyState from '@/components/ui/FdEmptyState.vue'
import FdErrorState from '@/components/ui/FdErrorState.vue'
import FdIconButton from '@/components/ui/FdIconButton.vue'
import FdLoadingSkeleton from '@/components/ui/FdLoadingSkeleton.vue'
import FdSegmented from '@/components/ui/FdSegmented.vue'
import { formatRelative } from '@/lib/format'
import { useAppStore } from '@/stores/app'
import { useInboxStore } from '@/stores/inbox'
import type { InboxEvent } from '@/types/ipc'

const inbox = useInboxStore()
const app = useAppStore()
const router = useRouter()
const { t } = useI18n()
const selected = ref<InboxEvent | null>(null)

const filters = computed(() => [
  { value: 'all', label: t('inboxPage.all') },
  { value: 'unread', label: t('inboxPage.unread') },
  { value: 'github', label: t('inboxPage.github') },
  { value: 'local', label: t('inboxPage.local') },
  { value: 'sync', label: t('inboxPage.sync') },
])

const groups = computed(() => {
  const today = new Date()
  today.setHours(0, 0, 0, 0)
  const yesterday = new Date(today)
  yesterday.setDate(today.getDate() - 1)
  const result = [
    { key: 'today', label: t('inboxPage.today'), events: [] as InboxEvent[] },
    { key: 'yesterday', label: t('inboxPage.yesterday'), events: [] as InboxEvent[] },
    { key: 'earlier', label: t('inboxPage.earlier'), events: [] as InboxEvent[] },
  ]
  for (const event of inbox.events) {
    const date = new Date(event.createdAt)
    if (date >= today) result[0]!.events.push(event)
    else if (date >= yesterday) result[1]!.events.push(event)
    else result[2]!.events.push(event)
  }
  return result.filter((group) => group.events.length > 0)
})

async function open(event: InboxEvent): Promise<void> {
  selected.value = event
  if (!event.read) {
    await inbox.markRead(event, true)
  }
}

onMounted(() => {
  void inbox.load()
})
</script>

<template>
  <div class="fd-inbox-page">
    <PageHeader :title="t('pages.inbox.title')" :description="t('pages.inbox.description')">
      <template #actions>
        <FdButton variant="ghost" :disabled="inbox.unread === 0" @click="inbox.markAllRead()">
          {{ t('inboxPage.markAllRead') }}
        </FdButton>
      </template>
    </PageHeader>

    <PageToolbar>
      <template #left>
        <FdSegmented
          :model-value="inbox.filter"
          :options="filters"
          :label="t('inboxPage.filter')"
          @update:model-value="inbox.setFilter(String($event))"
        />
      </template>
      <template #right>
        <span class="fd-inbox-unread">{{ t('inboxPage.unreadCount', { count: inbox.unread }) }}</span>
      </template>
    </PageToolbar>

    <div class="fd-inbox-content">
      <div class="fd-inbox-list-wrap">
        <div v-if="inbox.loading" class="fd-inbox-state">
          <FdLoadingSkeleton :rows="7" :height="48" />
        </div>
        <div v-else-if="inbox.error" class="fd-inbox-state">
          <FdErrorState
            :title="t('inboxPage.loadError')"
            :message="inbox.error.message"
            :code="inbox.error.code"
            :detail="inbox.error.detail"
            @retry="inbox.load()"
          />
        </div>
        <FdEmptyState
          v-else-if="inbox.events.length === 0"
          :icon="InboxIcon"
          :title="t('inboxPage.empty')"
          :body="t('inboxPage.emptyBody')"
        />
        <div v-else class="fd-inbox-list">
          <section v-for="group in groups" :key="group.key" class="fd-inbox-group">
            <h2>{{ group.label }}</h2>
            <div
              v-for="event in group.events"
              :key="event.id"
              class="fd-inbox-row"
              :class="{ 'fd-inbox-row--unread': !event.read }"
              role="button"
              tabindex="0"
              @click="open(event)"
              @keydown.enter="open(event)"
            >
              <span class="fd-inbox-dot" :class="{ 'fd-inbox-dot--on': !event.read }" />
              <div class="fd-inbox-main">
                <div class="fd-inbox-title">
                  <strong>{{ event.title }}</strong>
                  <span class="fd-inbox-source">{{ event.source }}</span>
                </div>
                <span>{{ event.summary || event.entityTitle }}</span>
              </div>
              <span class="fd-inbox-key">{{ event.entityKey }}</span>
              <span class="fd-inbox-time">{{ formatRelative(event.createdAt, app.resolvedLocale) }}</span>
              <div class="fd-inbox-actions" @click.stop>
                <FdIconButton
                  :label="event.read ? t('inboxPage.markUnread') : t('inboxPage.markRead')"
                  @click="inbox.markRead(event, !event.read)"
                >
                  <Mail v-if="event.read" :size="14" />
                  <MailOpen v-else :size="14" />
                </FdIconButton>
                <FdIconButton :label="t('inboxPage.archive')" @click="inbox.archive(event)">
                  <Archive :size="14" />
                </FdIconButton>
              </div>
            </div>
          </section>
        </div>
      </div>

      <EntityDetailPane v-if="selected" :label="selected.entityKey" @close="selected = null">
        <header class="fd-inbox-detail-head">
          <div>
            <span class="fd-inbox-key">{{ selected.entityKey }} · {{ selected.source }}</span>
            <strong>{{ selected.title }}</strong>
          </div>
          <FdIconButton :label="t('common.close')" @click="selected = null">
            <X :size="16" />
          </FdIconButton>
        </header>
        <div class="fd-inbox-detail-body">
          <p>{{ selected.summary || selected.entityTitle }}</p>
          <dl>
            <div><dt>{{ t('inboxPage.entity') }}</dt><dd>{{ selected.entityTitle }}</dd></div>
            <div><dt>{{ t('inboxPage.occurred') }}</dt><dd>{{ formatRelative(selected.createdAt, app.resolvedLocale) }}</dd></div>
          </dl>
          <FdButton variant="secondary" @click="router.push('/my-issues')">
            {{ t('inboxPage.openIssue') }}
          </FdButton>
        </div>
      </EntityDetailPane>
    </div>
  </div>
</template>

<style scoped>
.fd-inbox-page { display: flex; flex-direction: column; width: 100%; height: 100%; min-width: 0; min-height: 0; overflow: hidden; }
.fd-inbox-unread { color: var(--fd-text-muted); font-size: var(--fd-text-11); }
.fd-inbox-content { display: flex; flex: 1; min-width: 0; min-height: 0; }
.fd-inbox-list-wrap { flex: 1; min-width: 0; overflow: auto; }
.fd-inbox-state { padding: var(--fd-space-6) var(--fd-content-pad); }
.fd-inbox-group h2 { position: sticky; top: 0; z-index: 1; margin: 0; padding: 8px var(--fd-content-pad) 4px; background: var(--fd-bg); color: var(--fd-text-muted); font-size: var(--fd-text-11); font-weight: var(--fd-weight-medium); text-transform: uppercase; }
.fd-inbox-row { display: grid; grid-template-columns: 10px minmax(0, 1fr) 90px 90px 60px; gap: var(--fd-space-2); align-items: center; min-height: 52px; padding: 4px var(--fd-content-pad); border-bottom: 1px solid var(--fd-border-subtle); cursor: pointer; }
.fd-inbox-row:hover { background: var(--fd-surface-hover); }
.fd-inbox-row--unread { background: color-mix(in srgb, var(--fd-accent) 5%, transparent); }
.fd-inbox-dot { width: 6px; height: 6px; border-radius: 50%; background: transparent; }
.fd-inbox-dot--on { background: var(--fd-accent); }
.fd-inbox-main { display: flex; flex-direction: column; gap: 1px; min-width: 0; }
.fd-inbox-title { display: flex; gap: 8px; align-items: baseline; }
.fd-inbox-title strong { font-size: var(--fd-text-12); font-weight: var(--fd-weight-medium); }
.fd-inbox-main > span { overflow: hidden; color: var(--fd-text-muted); font-size: var(--fd-text-11); text-overflow: ellipsis; white-space: nowrap; }
.fd-inbox-source { padding: 0 5px; border: 1px solid var(--fd-border-subtle); border-radius: 3px; color: var(--fd-text-muted); font-size: var(--fd-text-10); text-transform: uppercase; }
.fd-inbox-key { color: var(--fd-text-muted); font-family: var(--fd-font-mono); font-size: var(--fd-text-10); }
.fd-inbox-time { color: var(--fd-text-muted); font-size: var(--fd-text-11); white-space: nowrap; }
.fd-inbox-actions { display: flex; justify-content: flex-end; gap: 2px; opacity: 0; }
.fd-inbox-row:hover .fd-inbox-actions, .fd-inbox-row:focus-within .fd-inbox-actions { opacity: 1; }
.fd-inbox-detail-head { display: flex; align-items: flex-start; justify-content: space-between; gap: 8px; padding: var(--fd-space-3) var(--fd-space-4); border-bottom: 1px solid var(--fd-border-subtle); }
.fd-inbox-detail-head > div { display: flex; flex-direction: column; gap: 2px; min-width: 0; }
.fd-inbox-detail-head strong { font-size: var(--fd-text-15); }
.fd-inbox-detail-body { display: flex; flex-direction: column; gap: var(--fd-space-3); padding: var(--fd-space-4); }
.fd-inbox-detail-body p { margin: 0; color: var(--fd-text-secondary); font-size: var(--fd-text-12); line-height: var(--fd-line-body); }
.fd-inbox-detail-body dl { display: flex; flex-direction: column; gap: 6px; margin: 0; }
.fd-inbox-detail-body dt { color: var(--fd-text-muted); font-size: var(--fd-text-11); }
.fd-inbox-detail-body dd { margin: 0; color: var(--fd-text-secondary); font-size: var(--fd-text-11); }
</style>
