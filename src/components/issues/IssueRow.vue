<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'

import { formatRelative } from '@/lib/format'
import { useAppStore } from '@/stores/app'
import type { IssueSummary } from '@/types/ipc'

import IssueStatusBadge from './IssueStatusBadge.vue'
import IssueTypeIcon from './IssueTypeIcon.vue'
import LabelChip from './LabelChip.vue'
import PriorityIndicator from './PriorityIndicator.vue'

const props = withDefaults(
  defineProps<{
    issue: IssueSummary
    selected?: boolean
    compact?: boolean
  }>(),
  { selected: false, compact: false },
)

const emit = defineEmits<{
  (event: 'select', id: string): void
  (event: 'toggle', id: string): void
}>()

const app = useAppStore()
const { t } = useI18n()

const visibleLabels = computed(() => props.issue.labels.slice(0, 2))
const extraLabels = computed(() =>
  Math.max(0, props.issue.labels.length - visibleLabels.value.length),
)

function select(): void {
  emit('select', props.issue.id)
}
</script>

<template>
  <div
    class="fd-issue-row fd-issue-grid"
    :class="{ 'fd-issue-grid--compact': compact, 'fd-issue-row--selected': selected }"
    role="row"
    tabindex="0"
    :aria-label="`${issue.key} ${issue.title}`"
    data-testid="issue-row"
    @click="select"
    @keydown.enter.prevent="select"
  >
    <span class="fd-issue-cell fd-issue-cell--checkbox" role="cell" @click.stop>
      <input
        type="checkbox"
        :checked="selected"
        :aria-label="t('issues.selectIssue')"
        @change="emit('toggle', issue.id)"
      />
    </span>
    <span class="fd-issue-cell fd-issue-cell--key fd-mono" role="cell">
      {{ issue.key }}
    </span>
    <span class="fd-issue-cell fd-issue-cell--title" role="cell">
      <IssueTypeIcon :type="issue.issueType" />
      <span class="fd-issue-row__title">{{ issue.title }}</span>
      <span v-if="issue.subIssueCount" class="fd-issue-row__children">
        {{ issue.subIssueCount }}
      </span>
    </span>
    <span class="fd-issue-cell" role="cell">
      <IssueStatusBadge :status="issue.status" />
    </span>
    <span class="fd-issue-cell" role="cell">
      <PriorityIndicator :priority="issue.priority" />
    </span>
    <span class="fd-issue-cell fd-issue-cell--project" role="cell">
      <span v-if="issue.project" class="fd-truncate">{{ issue.project.name }}</span>
      <span v-else class="fd-issue-row__muted">—</span>
    </span>
    <span class="fd-issue-cell fd-issue-cell--labels" role="cell">
      <span class="fd-issue-row__labels">
        <LabelChip
          v-for="label in visibleLabels"
          :key="label.id"
          :label="label"
        />
        <span v-if="extraLabels" class="fd-issue-row__more-labels">
          +{{ extraLabels }}
        </span>
        <span v-if="!issue.labels.length" class="fd-issue-row__muted">—</span>
      </span>
    </span>
    <span class="fd-issue-cell fd-issue-cell--updated" role="cell">
      {{ formatRelative(issue.updatedAt, app.resolvedLocale) }}
    </span>
  </div>
</template>

<style scoped>
.fd-issue-row {
  min-height: var(--fd-row-height);
  padding: 0 var(--fd-content-pad);
  border-bottom: 1px solid var(--fd-border-subtle);
  color: var(--fd-text-secondary);
  cursor: default;
}

.fd-issue-row:hover {
  background: var(--fd-surface-hover);
}

.fd-issue-row--selected {
  background: var(--fd-surface-selected);
}

.fd-issue-row:focus-visible {
  outline: 2px solid var(--fd-accent);
  outline-offset: -2px;
}

.fd-issue-cell {
  min-width: 0;
  overflow: hidden;
  font-size: var(--fd-text-12);
}

.fd-issue-cell--checkbox {
  display: flex;
  align-items: center;
}

.fd-issue-cell--checkbox input {
  width: 14px;
  height: 14px;
  accent-color: var(--fd-accent);
}

.fd-issue-cell--key {
  color: var(--fd-text-muted);
  font-size: var(--fd-text-11);
}

.fd-issue-cell--title {
  display: flex;
  align-items: center;
  gap: 6px;
  color: var(--fd-text);
}

.fd-issue-row__title {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.fd-issue-row__children {
  display: inline-flex;
  flex: none;
  align-items: center;
  height: 16px;
  padding: 0 4px;
  border-radius: 999px;
  background: var(--fd-surface-selected);
  color: var(--fd-text-muted);
  font-size: var(--fd-text-11);
}

.fd-issue-row__labels {
  display: flex;
  align-items: center;
  gap: 3px;
  overflow: hidden;
}

.fd-issue-row__more-labels {
  flex: none;
  color: var(--fd-text-muted);
  font-size: var(--fd-text-11);
}

.fd-issue-row__muted {
  color: var(--fd-text-muted);
}

.fd-issue-cell--updated {
  color: var(--fd-text-muted);
  font-size: var(--fd-text-11);
  white-space: nowrap;
}
</style>
