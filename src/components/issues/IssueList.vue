<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'

import type { IssueSummary } from '@/types/ipc'

import IssueRow from './IssueRow.vue'

const props = withDefaults(
  defineProps<{
    issues: IssueSummary[]
    selectedIds: string[]
    compact?: boolean
  }>(),
  { compact: false },
)

const emit = defineEmits<{
  (event: 'select', id: string): void
  (event: 'toggle', id: string): void
  (event: 'toggle-all'): void
}>()

const { t } = useI18n()

const allSelected = computed(
  () =>
    props.issues.length > 0 &&
    props.issues.every((issue) => props.selectedIds.includes(issue.id)),
)
</script>

<template>
  <div class="fd-issue-list" role="table" :aria-label="t('nav.myIssues')">
    <div
      class="fd-issue-list__head fd-issue-grid"
      :class="{ 'fd-issue-grid--compact': compact }"
      role="row"
    >
      <span class="fd-issue-list__checkbox" role="columnheader">
        <input
          type="checkbox"
          :checked="allSelected"
          :aria-label="t('issues.selectAll')"
          @change="emit('toggle-all')"
        />
      </span>
      <span role="columnheader">{{ t('issues.columns.issue') }}</span>
      <span role="columnheader">{{ t('issues.columns.title') }}</span>
      <span role="columnheader">{{ t('issues.columns.status') }}</span>
      <span role="columnheader">{{ t('issues.columns.priority') }}</span>
      <span class="fd-issue-cell--project" role="columnheader">
        {{ t('issues.columns.project') }}
      </span>
      <span class="fd-issue-cell--labels" role="columnheader">
        {{ t('issues.columns.labels') }}
      </span>
      <span role="columnheader">{{ t('issues.columns.updated') }}</span>
    </div>
    <div class="fd-issue-list__body" role="rowgroup">
      <IssueRow
        v-for="issue in issues"
        :key="issue.id"
        :issue="issue"
        :selected="selectedIds.includes(issue.id)"
        :compact="compact"
        @select="emit('select', $event)"
        @toggle="emit('toggle', $event)"
      />
    </div>
  </div>
</template>

<style scoped>
.fd-issue-list {
  display: flex;
  flex-direction: column;
  min-height: 0;
}

.fd-issue-list__head {
  position: sticky;
  top: 0;
  z-index: 2;
  flex: none;
  min-height: 32px;
  padding: 0 var(--fd-content-pad);
  border-bottom: 1px solid var(--fd-border);
  background: var(--fd-bg);
  color: var(--fd-text-muted);
  font-size: var(--fd-text-11);
}

.fd-issue-list__checkbox input {
  width: 14px;
  height: 14px;
  accent-color: var(--fd-accent);
}

.fd-issue-list__body {
  display: flex;
  flex-direction: column;
}
</style>
