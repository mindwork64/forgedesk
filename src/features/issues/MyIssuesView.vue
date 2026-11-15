<script setup lang="ts">
import { ChevronDown, CircleDot, Plus } from '@lucide/vue'
import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRouter } from 'vue-router'

import PageHeader from '@/components/layout/PageHeader.vue'
import PageToolbar from '@/components/layout/PageToolbar.vue'
import FdButton from '@/components/ui/FdButton.vue'
import FdEmptyState from '@/components/ui/FdEmptyState.vue'

const { t } = useI18n()
const router = useRouter()

const views = computed(() => [
  { key: 'list', label: t('pages.myIssues.list') },
  { key: 'board', label: t('pages.myIssues.board') },
  { key: 'table', label: t('pages.myIssues.table') },
  { key: 'calendar', label: t('pages.myIssues.calendar') },
  { key: 'timeline', label: t('pages.myIssues.timeline') },
])

const activeView = ref('list')
const unavailable = computed(() => t('pages.myIssues.toolbarUnavailable'))

const toolbarActions = computed(() => [
  t('pages.myIssues.filter'),
  t('pages.myIssues.group'),
  t('pages.myIssues.sort'),
  t('pages.myIssues.savedView'),
])
</script>

<template>
  <div class="fd-page" data-testid="page-content">
    <PageHeader
      :title="t('pages.myIssues.title')"
      :description="t('pages.myIssues.description')"
    >
      <template #actions>
        <span v-tooltip="unavailable">
          <FdButton variant="primary" disabled>
            <Plus :size="14" aria-hidden="true" />
            {{ t('pages.myIssues.newIssue') }}
          </FdButton>
        </span>
      </template>
    </PageHeader>

    <PageToolbar>
      <template #left>
        <div class="fd-view-switcher" role="group" :aria-label="t('pages.myIssues.list')">
          <span
            v-for="view in views"
            :key="view.key"
            v-tooltip="view.key === 'list' ? false : unavailable"
          >
            <button
              type="button"
              class="fd-view-switcher__tab"
              :class="{ 'fd-view-switcher__tab--active': activeView === view.key }"
              :disabled="view.key !== 'list'"
              :aria-pressed="activeView === view.key"
            >
              {{ view.label }}
            </button>
          </span>
        </div>
      </template>
      <template #right>
        <span
          v-for="action in toolbarActions"
          :key="action"
          v-tooltip="unavailable"
        >
          <FdButton variant="ghost" size="sm" disabled>
            {{ action }}
            <ChevronDown :size="12" aria-hidden="true" />
          </FdButton>
        </span>
      </template>
    </PageToolbar>

    <div class="fd-page__body">
      <FdEmptyState
        :icon="CircleDot"
        :title="t('pages.myIssues.emptyTitle')"
        :body="t('pages.myIssues.emptyBody')"
      >
        <FdButton variant="secondary" @click="router.push('/settings/storage')">
          {{ t('pages.placeholder.openStorage') }}
        </FdButton>
      </FdEmptyState>
    </div>
  </div>
</template>

<style scoped>
.fd-page {
  display: flex;
  flex-direction: column;
  min-height: 100%;
}

.fd-page__body {
  display: flex;
  flex: 1;
  align-items: flex-start;
  justify-content: center;
  padding: var(--fd-space-7) var(--fd-content-pad);
}

.fd-view-switcher {
  display: inline-flex;
  align-items: center;
  gap: 2px;
}

.fd-view-switcher__tab {
  height: 28px;
  padding: 0 9px;
  border: 0;
  border-radius: var(--fd-radius-control);
  background: transparent;
  color: var(--fd-text-secondary);
  font-size: var(--fd-text-12);
  font-weight: var(--fd-weight-medium);
}

.fd-view-switcher__tab:hover:not(:disabled) {
  background: var(--fd-surface-hover);
  color: var(--fd-text);
}

.fd-view-switcher__tab--active {
  background: var(--fd-surface-selected);
  color: var(--fd-text);
}

.fd-view-switcher__tab:disabled {
  cursor: not-allowed;
  opacity: 0.45;
}
</style>
