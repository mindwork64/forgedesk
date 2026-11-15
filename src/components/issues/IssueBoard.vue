<script setup lang="ts">
import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'

import { useIssueStore } from '@/stores/issues'
import type { IssueSummary, StatusSummary } from '@/types/ipc'

import IssueStatusBadge from './IssueStatusBadge.vue'
import LabelChip from './LabelChip.vue'
import PriorityIndicator from './PriorityIndicator.vue'

defineProps<{
  columns: Array<{ status: StatusSummary; issues: IssueSummary[] }>
}>()

const emit = defineEmits<{ (event: 'select', id: string): void }>()
const issueStore = useIssueStore()
const { t } = useI18n()

const draggingId = ref<string | null>(null)
const dragOverStatus = ref<string | null>(null)

const activeIssue = computed(() =>
  draggingId.value
    ? issueStore.issues.find((issue) => issue.id === draggingId.value) ?? null
    : null,
)

function onDragStart(event: DragEvent, issue: IssueSummary): void {
  draggingId.value = issue.id
  if (event.dataTransfer) {
    event.dataTransfer.effectAllowed = 'move'
    event.dataTransfer.setData('text/plain', issue.id)
  }
}

function onDragEnd(): void {
  draggingId.value = null
  dragOverStatus.value = null
}

async function onDrop(statusId: string): Promise<void> {
  const id = draggingId.value
  const issue = activeIssue.value
  draggingId.value = null
  dragOverStatus.value = null
  if (!id || !issue || issue.status.id === statusId) {
    return
  }
  await issueStore.updateIssue(id, { statusId })
}
</script>

<template>
  <div class="fd-board" role="region" :aria-label="t('nav.myIssues')">
    <section
      v-for="column in columns"
      :key="column.status.id"
      class="fd-board__column"
      :class="{ 'fd-board__column--over': dragOverStatus === column.status.id }"
      @dragover.prevent="dragOverStatus = column.status.id"
      @dragleave="dragOverStatus = null"
      @drop.prevent="onDrop(column.status.id)"
    >
      <header class="fd-board__header">
        <IssueStatusBadge :status="column.status" />
        <span class="fd-board__count">{{ column.issues.length }}</span>
      </header>
      <div class="fd-board__cards">
        <article
          v-for="issue in column.issues"
          :key="issue.id"
          class="fd-board__card"
          draggable="true"
          tabindex="0"
          :aria-label="`${issue.key} ${issue.title}`"
          @click="emit('select', issue.id)"
          @keydown.enter.prevent="emit('select', issue.id)"
          @dragstart="onDragStart($event, issue)"
          @dragend="onDragEnd"
        >
          <div class="fd-board__card-top">
            <span class="fd-board__key fd-mono">{{ issue.key }}</span>
            <PriorityIndicator :priority="issue.priority" />
          </div>
          <p class="fd-board__title">{{ issue.title }}</p>
          <div class="fd-board__meta">
            <LabelChip
              v-for="label in issue.labels.slice(0, 2)"
              :key="label.id"
              :label="label"
            />
            <span v-if="issue.project" class="fd-board__project">
              {{ issue.project.name }}
            </span>
            <span v-if="issue.estimate !== null" class="fd-board__estimate">
              {{ issue.estimate }}
            </span>
          </div>
        </article>
        <p v-if="column.issues.length === 0" class="fd-board__empty">
          {{ t('issues.noIssues') }}
        </p>
      </div>
    </section>
  </div>
</template>

<style scoped>
.fd-board {
  display: flex;
  gap: var(--fd-space-3);
  width: 100%;
  max-width: 100%;
  height: 100%;
  padding: var(--fd-space-3) var(--fd-content-pad) var(--fd-space-5);
  overflow-x: auto;
  overflow-y: hidden;
}

.fd-board__column {
  display: flex;
  flex: 0 0 272px;
  flex-direction: column;
  min-height: 0;
  border-radius: var(--fd-radius-panel);
  background: var(--fd-surface);
  border: 1px solid var(--fd-border-subtle);
}

.fd-board__column--over {
  border-color: var(--fd-accent);
}

.fd-board__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--fd-space-2);
  padding: 8px 10px;
  border-bottom: 1px solid var(--fd-border-subtle);
}

.fd-board__count {
  color: var(--fd-text-muted);
  font-size: var(--fd-text-11);
}

.fd-board__cards {
  display: flex;
  flex: 1;
  flex-direction: column;
  gap: 6px;
  min-height: 0;
  padding: 8px;
  overflow-y: auto;
}

.fd-board__card {
  display: flex;
  flex-direction: column;
  gap: 5px;
  padding: 8px 10px;
  border: 1px solid var(--fd-border);
  border-radius: var(--fd-radius-control);
  background: var(--fd-surface-raised);
  cursor: pointer;
}

.fd-board__card:hover {
  border-color: var(--fd-text-muted);
}

.fd-board__card:focus-visible {
  outline: 2px solid var(--fd-accent);
  outline-offset: 1px;
}

.fd-board__card-top {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--fd-space-2);
}

.fd-board__key {
  color: var(--fd-text-muted);
  font-size: var(--fd-text-11);
}

.fd-board__title {
  display: -webkit-box;
  overflow: hidden;
  color: var(--fd-text);
  font-size: var(--fd-text-12);
  line-height: var(--fd-line-body);
  -webkit-box-orient: vertical;
  -webkit-line-clamp: 2;
}

.fd-board__meta {
  display: flex;
  flex-wrap: wrap;
  gap: 3px;
  align-items: center;
}

.fd-board__project,
.fd-board__estimate {
  color: var(--fd-text-muted);
  font-size: var(--fd-text-11);
}

.fd-board__empty {
  padding: var(--fd-space-3);
  color: var(--fd-text-muted);
  font-size: var(--fd-text-11);
  text-align: center;
}
</style>
