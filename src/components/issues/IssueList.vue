<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'

import { issueTypeLabel, priorityLabel, statusLabel } from '@/lib/issue'
import type { IssueSummary } from '@/types/ipc'

import IssueRow from './IssueRow.vue'

const props = withDefaults(
  defineProps<{
    issues: IssueSummary[]
    selectedIds: string[]
    compact?: boolean
    groupBy?: string | null
  }>(),
  { compact: false, groupBy: null },
)

const emit = defineEmits<{
  (event: 'select', id: string): void
  (event: 'toggle', id: string): void
  (event: 'toggle-all'): void
}>()

const { t, te } = useI18n()

const ROW_HEIGHT = 40
const OVERSCAN = 8
const VIRTUAL_THRESHOLD = 100

const body = ref<HTMLElement | null>(null)
const scrollTop = ref(0)
const viewportHeight = ref(0)
let observer: ResizeObserver | null = null

const virtual = computed(
  () => !props.groupBy && props.issues.length > VIRTUAL_THRESHOLD,
)
const allSelected = computed(
  () =>
    props.issues.length > 0 &&
    props.issues.every((issue) => props.selectedIds.includes(issue.id)),
)

interface IssueGroup {
  key: string
  label: string
  issues: IssueSummary[]
}

const PRIORITY_ORDER: Record<string, number> = {
  none: 0,
  low: 1,
  medium: 2,
  high: 3,
  urgent: 4,
}

function groupMeta(issue: IssueSummary): { key: string; label: string; order: number } {
  switch (props.groupBy) {
    case 'status':
      return {
        key: issue.status.id,
        label: statusLabel(issue.status, t, te),
        order: issue.status.position,
      }
    case 'priority':
      return {
        key: issue.priority,
        label: priorityLabel(issue.priority, t),
        order: PRIORITY_ORDER[issue.priority] ?? 0,
      }
    case 'project':
      return {
        key: issue.project?.id ?? 'none',
        label: issue.project?.name ?? t('issues.noProject'),
        order: issue.project ? 0 : 1,
      }
    case 'type':
      return {
        key: issue.issueType,
        label: issueTypeLabel(issue.issueType, t),
        order: 0,
      }
    default:
      return { key: 'all', label: '', order: 0 }
  }
}

const groups = computed<IssueGroup[]>(() => {
  if (!props.groupBy) {
    return []
  }
  const map = new Map<string, IssueGroup & { order: number }>()
  for (const issue of props.issues) {
    const meta = groupMeta(issue)
    const group = map.get(meta.key)
    if (group) {
      group.issues.push(issue)
    } else {
      map.set(meta.key, {
        key: meta.key,
        label: meta.label,
        order: meta.order,
        issues: [issue],
      })
    }
  }
  return [...map.values()]
    .sort((a, b) => a.order - b.order || a.label.localeCompare(b.label))
    .map(({ order: _order, ...group }) => group)
})

const visibleRows = computed(() => {
  if (!virtual.value) {
    return props.issues.map((issue, index) => ({ issue, index }))
  }
  const start = Math.max(
    0,
    Math.floor(scrollTop.value / ROW_HEIGHT) - OVERSCAN,
  )
  const end = Math.min(
    props.issues.length,
    Math.ceil((scrollTop.value + viewportHeight.value) / ROW_HEIGHT) +
      OVERSCAN,
  )
  return props.issues
    .slice(start, end)
    .map((issue, offset) => ({ issue, index: start + offset }))
})

function measure(): void {
  viewportHeight.value = body.value?.clientHeight ?? 0
}

function onScroll(): void {
  scrollTop.value = body.value?.scrollTop ?? 0
}

onMounted(() => {
  measure()
  if (typeof ResizeObserver !== 'undefined' && body.value) {
    observer = new ResizeObserver(measure)
    observer.observe(body.value)
  }
})

onBeforeUnmount(() => {
  observer?.disconnect()
  observer = null
})
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

    <div
      ref="body"
      class="fd-issue-list__body"
      :class="{ 'fd-issue-list__body--virtual': virtual }"
      role="rowgroup"
      @scroll="onScroll"
    >
      <div
        v-if="virtual"
        class="fd-issue-list__spacer"
        :style="{ height: `${issues.length * ROW_HEIGHT}px` }"
      >
        <IssueRow
          v-for="row in visibleRows"
          :key="row.issue.id"
          class="fd-issue-list__virtual-row"
          :style="{ top: `${row.index * ROW_HEIGHT}px` }"
          :issue="row.issue"
          :selected="selectedIds.includes(row.issue.id)"
          :compact="compact"
          @select="emit('select', $event)"
          @toggle="emit('toggle', $event)"
        />
      </div>
      <template v-else-if="groupBy">
        <section
          v-for="group in groups"
          :key="group.key"
          class="fd-issue-group"
        >
          <div class="fd-issue-group__head">
            <span>{{ group.label }}</span>
            <span class="fd-issue-group__count">{{ group.issues.length }}</span>
          </div>
          <IssueRow
            v-for="issue in group.issues"
            :key="issue.id"
            :issue="issue"
            :selected="selectedIds.includes(issue.id)"
            :compact="compact"
            @select="emit('select', $event)"
            @toggle="emit('toggle', $event)"
          />
        </section>
      </template>
      <template v-else>
        <IssueRow
          v-for="issue in issues"
          :key="issue.id"
          :issue="issue"
          :selected="selectedIds.includes(issue.id)"
          :compact="compact"
          @select="emit('select', $event)"
          @toggle="emit('toggle', $event)"
        />
      </template>
    </div>
  </div>
</template>

<style scoped>
.fd-issue-list {
  display: flex;
  flex: 1;
  flex-direction: column;
  height: 100%;
  min-height: 0;
}

.fd-issue-list__head {
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
  flex: 1;
  min-height: 0;
  overflow-y: auto;
}

.fd-issue-list__body--virtual {
  position: relative;
}

.fd-issue-list__spacer {
  position: relative;
  width: 100%;
}

.fd-issue-list__virtual-row {
  position: absolute;
  right: 0;
  left: 0;
  height: 40px;
}

.fd-issue-group__head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 32px;
  padding: 0 var(--fd-content-pad);
  border-bottom: 1px solid var(--fd-border-subtle);
  background: var(--fd-surface);
  color: var(--fd-text-secondary);
  font-size: var(--fd-text-11);
  font-weight: var(--fd-weight-semibold);
  text-transform: uppercase;
  letter-spacing: 0.04em;
}

.fd-issue-group__count {
  color: var(--fd-text-muted);
  font-weight: var(--fd-weight-regular);
}
</style>
