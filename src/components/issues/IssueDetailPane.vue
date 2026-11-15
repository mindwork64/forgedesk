<script setup lang="ts">
import {
  Archive,
  Copy,
  Link2,
  MessageSquare,
  Plus,
  X,
} from '@lucide/vue'
import { computed, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'

import FdBadge from '@/components/ui/FdBadge.vue'
import FdButton from '@/components/ui/FdButton.vue'
import FdConfirmDialog from '@/components/ui/FdConfirmDialog.vue'
import FdIconButton from '@/components/ui/FdIconButton.vue'
import FdInput from '@/components/ui/FdInput.vue'
import FdSelect from '@/components/ui/FdSelect.vue'
import FdTextarea from '@/components/ui/FdTextarea.vue'
import { formatDateTime, formatRelative } from '@/lib/format'
import {
  activityText,
  issueTypeLabel,
  priorityLabel,
  relationKindLabel,
  statusLabel,
} from '@/lib/issue'
import { useAppStore } from '@/stores/app'
import { useIssueStore } from '@/stores/issues'
import type { IssueType, Priority, RelationInputKind } from '@/types/ipc'

import EntityDetailPane from './EntityDetailPane.vue'
import IssueStatusBadge from './IssueStatusBadge.vue'
import LabelChip from './LabelChip.vue'

const emit = defineEmits<{
  (event: 'create-subissue', parentId: string, parentKey: string): void
}>()

const issueStore = useIssueStore()
const app = useAppStore()
const { t, te } = useI18n()

const issue = computed(() => issueStore.detail?.summary ?? null)
const detail = computed(() => issueStore.detail)

const titleDraft = ref('')
const descriptionDraft = ref('')
const descriptionEditing = ref(false)
const descriptionSaving = ref(false)
const estimateDraft = ref('')
const labelDraft = ref('')
const newComment = ref('')
const commentBusy = ref(false)
const relationKind = ref<RelationInputKind>('related')
const relationIssueId = ref('')
const relationBusy = ref(false)
const archiveOpen = ref(false)
const archiveBusy = ref(false)

watch(
  () => issue.value?.id,
  () => {
    titleDraft.value = issue.value?.title ?? ''
    descriptionDraft.value = detail.value?.description ?? ''
    descriptionEditing.value = false
    estimateDraft.value =
      issue.value?.estimate === null || issue.value?.estimate === undefined
        ? ''
        : String(issue.value.estimate)
    labelDraft.value = ''
    newComment.value = ''
    relationIssueId.value = ''
  },
  { immediate: true },
)

watch(
  () => issue.value?.title,
  (value) => {
    if (!descriptionEditing.value) {
      titleDraft.value = value ?? ''
    }
  },
)

const statusOptions = computed(() =>
  issueStore.statuses.map((status) => ({
    value: status.id,
    label: statusLabel(status, t, te),
  })),
)
const priorityOptions = computed(() =>
  (['none', 'low', 'medium', 'high', 'urgent'] as Priority[]).map((value) => ({
    value,
    label: priorityLabel(value, t),
  })),
)
const typeOptions = computed(() =>
  (['bug', 'feature', 'task', 'improvement', 'chore', 'research'] as IssueType[]).map(
    (value) => ({ value, label: issueTypeLabel(value, t) }),
  ),
)
const relationOptions = computed(() =>
  issueStore.issues
    .filter((candidate) => candidate.id !== issue.value?.id)
    .map((candidate) => ({
      id: candidate.id,
      label: `${candidate.key} · ${candidate.title}`,
    })),
)
const relationKindOptions = computed(() => [
  { value: 'blocks', label: t('relation.blocks') },
  { value: 'related', label: t('relation.related') },
  { value: 'duplicate', label: t('relation.duplicate') },
])

const sourceLabel = computed(() => {
  switch (issue.value?.source) {
    case 'github':
      return t('issues.sourceGithub')
    case 'gitlab':
      return t('issues.sourceGitlab')
    case 'gitea':
      return t('issues.sourceGitea')
    default:
      return t('issues.sourceLocal')
  }
})

const labelNames = computed(
  () => issue.value?.labels.map((label) => label.name) ?? [],
)

async function saveTitle(): Promise<void> {
  const current = issue.value
  if (!current) {
    return
  }
  const trimmed = titleDraft.value.trim()
  if (!trimmed) {
    titleDraft.value = current.title
    return
  }
  if (trimmed === current.title) {
    return
  }
  const ok = await issueStore.updateIssue(current.id, { title: trimmed })
  if (!ok) {
    titleDraft.value = current.title
  }
}

function startDescriptionEdit(): void {
  descriptionDraft.value = detail.value?.description ?? ''
  descriptionEditing.value = true
}

async function saveDescription(): Promise<void> {
  const current = issue.value
  if (!current) {
    return
  }
  descriptionSaving.value = true
  const ok = await issueStore.updateIssue(current.id, {
    description: descriptionDraft.value,
  })
  descriptionSaving.value = false
  if (ok) {
    descriptionEditing.value = false
  }
}

async function changeStatus(statusId: string): Promise<void> {
  if (!issue.value || statusId === issue.value.status.id) {
    return
  }
  await issueStore.updateIssue(issue.value.id, { statusId })
}

async function changePriority(value: string): Promise<void> {
  if (!issue.value || value === issue.value.priority) {
    return
  }
  await issueStore.updateIssue(issue.value.id, { priority: value as Priority })
}

async function changeType(value: string): Promise<void> {
  if (!issue.value || value === issue.value.issueType) {
    return
  }
  await issueStore.updateIssue(issue.value.id, { issueType: value as IssueType })
}

async function changeProject(value: string): Promise<void> {
  if (!issue.value) {
    return
  }
  if (!value) {
    await issueStore.updateIssue(issue.value.id, {
      clearFields: ['projectId'],
    })
    return
  }
  await issueStore.updateIssue(issue.value.id, { projectId: value })
}

async function saveEstimate(): Promise<void> {
  const current = issue.value
  if (!current) {
    return
  }
  const raw = estimateDraft.value.trim()
  if (!raw) {
    if (current.estimate !== null) {
      await issueStore.updateIssue(current.id, { clearFields: ['estimate'] })
    }
    return
  }
  const value = Number(raw)
  if (!Number.isFinite(value) || value < 0) {
    estimateDraft.value = current.estimate === null ? '' : String(current.estimate)
    return
  }
  if (value === current.estimate) {
    return
  }
  await issueStore.updateIssue(current.id, { estimate: value })
}

async function saveDate(
  field: 'startDate' | 'dueDate',
  value: string,
): Promise<void> {
  const current = issue.value
  if (!current) {
    return
  }
  if (!value) {
    await issueStore.updateIssue(current.id, { clearFields: [field] })
    return
  }
  await issueStore.updateIssue(current.id, { [field]: value })
}

async function addLabels(): Promise<void> {
  const current = issue.value
  if (!current) {
    return
  }
  const additions = labelDraft.value
    .split(',')
    .map((value) => value.trim())
    .filter(Boolean)
  if (additions.length === 0) {
    return
  }
  const names = [...labelNames.value]
  for (const addition of additions) {
    if (!names.some((name) => name.toLowerCase() === addition.toLowerCase())) {
      names.push(addition)
    }
  }
  const ok = await issueStore.updateIssue(current.id, { labelNames: names })
  if (ok) {
    labelDraft.value = ''
  }
}

async function removeLabel(name: string): Promise<void> {
  const current = issue.value
  if (!current) {
    return
  }
  await issueStore.updateIssue(current.id, {
    labelNames: labelNames.value.filter((label) => label !== name),
  })
}

async function clearParent(): Promise<void> {
  if (!issue.value) {
    return
  }
  await issueStore.updateIssue(issue.value.id, {
    clearFields: ['parentId'],
  })
}

async function addRelation(): Promise<void> {
  if (!issue.value || !relationIssueId.value || relationBusy.value) {
    return
  }
  relationBusy.value = true
  const ok = await issueStore.addRelation(
    issue.value.id,
    relationIssueId.value,
    relationKind.value,
  )
  relationBusy.value = false
  if (ok) {
    relationIssueId.value = ''
  }
}

async function removeRelation(relationId: string): Promise<void> {
  if (!issue.value) {
    return
  }
  await issueStore.removeRelation(issue.value.id, relationId)
}

async function submitComment(): Promise<void> {
  if (!issue.value || commentBusy.value || !newComment.value.trim()) {
    return
  }
  commentBusy.value = true
  const ok = await issueStore.addComment(issue.value.id, newComment.value)
  commentBusy.value = false
  if (ok) {
    newComment.value = ''
  }
}

async function confirmArchive(): Promise<void> {
  if (!issue.value || archiveBusy.value) {
    return
  }
  archiveBusy.value = true
  const ok = await issueStore.archiveIssue(issue.value.id, true)
  archiveBusy.value = false
  if (ok) {
    archiveOpen.value = false
  }
}

async function duplicate(): Promise<void> {
  if (!issue.value) {
    return
  }
  await issueStore.duplicateIssue(issue.value.id)
}

function createSubIssue(): void {
  if (issue.value) {
    emit('create-subissue', issue.value.id, issue.value.key)
  }
}
</script>

<template>
  <EntityDetailPane
    v-if="issue && detail"
    :label="issue.key"
    @close="issueStore.closeDetail()"
  >
    <header class="fd-detail-header">
      <div class="fd-detail-header__identity">
        <span class="fd-detail-header__key fd-mono">{{ issue.key }}</span>
        <FdBadge tone="neutral">{{ sourceLabel }}</FdBadge>
        <span
          v-if="issue.archivedAt"
          class="fd-detail-header__archived"
        >{{ t('issues.archived') }}</span>
      </div>
      <div class="fd-detail-header__actions">
        <FdIconButton :label="t('issues.duplicate')" @click="duplicate">
          <Copy :size="15" />
        </FdIconButton>
        <FdIconButton :label="t('issues.archive')" @click="archiveOpen = true">
          <Archive :size="15" />
        </FdIconButton>
        <FdIconButton :label="t('issues.close')" @click="issueStore.closeDetail()">
          <X :size="16" />
        </FdIconButton>
      </div>
    </header>

    <div class="fd-detail-body">
      <input
        v-model="titleDraft"
        class="fd-detail-title"
        :aria-label="t('issues.title')"
        data-testid="issue-detail-title"
        @blur="saveTitle"
        @keydown.enter.prevent="($event.target as HTMLInputElement).blur()"
      />

      <p class="fd-detail-subtitle">
        {{ issueTypeLabel(issue.issueType, t) }} ·
        {{ t('issues.columns.updated') }}
        <span :title="formatDateTime(issue.updatedAt, app.resolvedLocale)">
          {{ formatRelative(issue.updatedAt, app.resolvedLocale) }}
        </span>
      </p>

      <section class="fd-detail-section">
        <div v-if="!descriptionEditing" class="fd-detail-description">
          <p v-if="detail.description" class="fd-detail-description__text">
            {{ detail.description }}
          </p>
          <p v-else class="fd-detail-description__empty">
            {{ t('issues.noDescription') }}
          </p>
          <button
            type="button"
            class="fd-detail-description__edit"
            @click="startDescriptionEdit"
          >
            {{ t('common.edit') }}
          </button>
        </div>
        <div v-else class="fd-detail-description-editor">
          <FdTextarea
            v-model="descriptionDraft"
            :rows="6"
            autofocus
            :placeholder="t('issues.descriptionPlaceholder')"
            @keydown.ctrl.enter.prevent="saveDescription"
            @keydown.meta.enter.prevent="saveDescription"
          />
          <div class="fd-detail-description-editor__actions">
            <FdButton
              variant="ghost"
              size="sm"
              :disabled="descriptionSaving"
              @click="descriptionEditing = false"
            >
              {{ t('common.cancel') }}
            </FdButton>
            <FdButton
              variant="primary"
              size="sm"
              :loading="descriptionSaving"
              @click="saveDescription"
            >
              {{ t('common.save') }}
            </FdButton>
          </div>
        </div>
      </section>

      <section class="fd-detail-section">
        <h3 class="fd-detail-section__title">{{ t('issues.properties') }}</h3>
        <div class="fd-property-grid">
          <span class="fd-property-grid__label">{{ t('issues.status') }}</span>
          <FdSelect
            :model-value="issue.status.id"
            :aria-label="t('issues.status')"
            @update:model-value="changeStatus"
          >
            <option
              v-for="option in statusOptions"
              :key="option.value"
              :value="option.value"
            >
              {{ option.label }}
            </option>
          </FdSelect>

          <span class="fd-property-grid__label">{{ t('issues.priority') }}</span>
          <FdSelect
            :model-value="issue.priority"
            :aria-label="t('issues.priority')"
            @update:model-value="changePriority"
          >
            <option
              v-for="option in priorityOptions"
              :key="option.value"
              :value="option.value"
            >
              {{ option.label }}
            </option>
          </FdSelect>

          <span class="fd-property-grid__label">{{ t('issues.type') }}</span>
          <FdSelect
            :model-value="issue.issueType"
            :aria-label="t('issues.type')"
            @update:model-value="changeType"
          >
            <option
              v-for="option in typeOptions"
              :key="option.value"
              :value="option.value"
            >
              {{ option.label }}
            </option>
          </FdSelect>

          <span class="fd-property-grid__label">{{ t('issues.project') }}</span>
          <FdSelect
            :model-value="issue.project?.id ?? ''"
            :aria-label="t('issues.project')"
            @update:model-value="changeProject"
          >
            <option value="">{{ t('issues.noProject') }}</option>
            <option
              v-for="project in issueStore.projects"
              :key="project.id"
              :value="project.id"
            >
              {{ project.name }}
            </option>
          </FdSelect>

          <span class="fd-property-grid__label">{{ t('issues.estimate') }}</span>
          <FdInput
            v-model="estimateDraft"
            type="number"
            :placeholder="t('issues.noEstimate')"
            :aria-label="t('issues.estimate')"
            @blur="saveEstimate"
            @keydown.enter="saveEstimate"
          />

          <span class="fd-property-grid__label">{{ t('issues.startDate') }}</span>
          <FdInput
            :model-value="issue.startDate ?? ''"
            type="date"
            :aria-label="t('issues.startDate')"
            @update:model-value="saveDate('startDate', $event)"
          />

          <span class="fd-property-grid__label">{{ t('issues.dueDate') }}</span>
          <FdInput
            :model-value="issue.dueDate ?? ''"
            type="date"
            :aria-label="t('issues.dueDate')"
            @update:model-value="saveDate('dueDate', $event)"
          />
        </div>

        <div class="fd-detail-labels">
          <span class="fd-property-grid__label">{{ t('issues.labels') }}</span>
          <div class="fd-detail-labels__items">
            <span
              v-for="label in issue.labels"
              :key="label.id"
              class="fd-detail-labels__item"
            >
              <LabelChip :label="label" />
              <button
                type="button"
                class="fd-detail-labels__remove"
                :aria-label="`${t('common.remove')} ${label.name}`"
                @click="removeLabel(label.name)"
              >
                <X :size="11" />
              </button>
            </span>
            <FdInput
              v-model="labelDraft"
              class="fd-detail-labels__input"
              :placeholder="t('issues.labelsPlaceholder')"
              :aria-label="t('issues.labels')"
              @keydown.enter.prevent="addLabels"
            />
          </div>
        </div>
      </section>

      <section v-if="detail.parent" class="fd-detail-section">
        <h3 class="fd-detail-section__title">{{ t('issues.parentIssue') }}</h3>
        <div class="fd-detail-relation-row">
          <button
            type="button"
            class="fd-detail-issue-link"
            @click="issueStore.selectIssue(detail.parent.id)"
          >
            <span class="fd-mono">{{ detail.parent.key }}</span>
            {{ detail.parent.title }}
          </button>
          <FdIconButton :label="t('common.remove')" @click="clearParent">
            <X :size="13" />
          </FdIconButton>
        </div>
      </section>

      <section class="fd-detail-section">
        <div class="fd-detail-section__header">
          <h3 class="fd-detail-section__title">
            {{ t('issues.subIssues') }}
            <span v-if="detail.children.length" class="fd-detail-count">
              {{ detail.children.length }}
            </span>
          </h3>
          <FdButton variant="ghost" size="sm" @click="createSubIssue">
            <Plus :size="12" aria-hidden="true" />
            {{ t('issues.addSubIssue') }}
          </FdButton>
        </div>
        <div v-if="detail.children.length" class="fd-detail-list">
          <button
            v-for="child in detail.children"
            :key="child.id"
            type="button"
            class="fd-detail-issue-link"
            @click="issueStore.selectIssue(child.id)"
          >
            <IssueStatusBadge :status="child.status" />
            <span class="fd-mono">{{ child.key }}</span>
            <span class="fd-truncate">{{ child.title }}</span>
          </button>
        </div>
        <p v-else class="fd-detail-empty">{{ t('issues.noSubIssues') }}</p>
      </section>

      <section class="fd-detail-section">
        <div class="fd-detail-section__header">
          <h3 class="fd-detail-section__title">
            {{ t('issues.relations') }}
            <span v-if="detail.relations.length" class="fd-detail-count">
              {{ detail.relations.length }}
            </span>
          </h3>
        </div>
        <div v-if="detail.relations.length" class="fd-detail-list">
          <div
            v-for="relation in detail.relations"
            :key="relation.id"
            class="fd-detail-relation-row"
          >
            <span class="fd-detail-relation-kind">
              {{ relationKindLabel(relation.kind, t) }}
            </span>
            <button
              type="button"
              class="fd-detail-issue-link"
              @click="issueStore.selectIssue(relation.issue.id)"
            >
              <span class="fd-mono">{{ relation.issue.key }}</span>
              <span class="fd-truncate">{{ relation.issue.title }}</span>
            </button>
            <FdIconButton
              :label="t('issues.removeRelation')"
              @click="removeRelation(relation.id)"
            >
              <X :size="13" />
            </FdIconButton>
          </div>
        </div>
        <p v-else class="fd-detail-empty">{{ t('issues.noRelations') }}</p>

        <div class="fd-detail-add-relation">
          <FdSelect
            v-model="relationKind"
            :aria-label="t('issues.relationKind')"
            class="fd-detail-add-relation__kind"
          >
            <option
              v-for="option in relationKindOptions"
              :key="option.value"
              :value="option.value"
            >
              {{ option.label }}
            </option>
          </FdSelect>
          <FdSelect
            v-model="relationIssueId"
            :aria-label="t('issues.relatedIssue')"
            class="fd-detail-add-relation__issue"
          >
            <option value="">{{ t('issues.relatedIssue') }}</option>
            <option
              v-for="option in relationOptions"
              :key="option.id"
              :value="option.id"
            >
              {{ option.label }}
            </option>
          </FdSelect>
          <FdButton
            variant="secondary"
            size="sm"
            :disabled="!relationIssueId"
            :loading="relationBusy"
            @click="addRelation"
          >
            <Link2 :size="12" aria-hidden="true" />
            {{ t('common.add') }}
          </FdButton>
        </div>
      </section>

      <section class="fd-detail-section">
        <h3 class="fd-detail-section__title">{{ t('issues.activity') }}</h3>
        <div v-if="detail.activity.length" class="fd-detail-activity">
          <div
            v-for="event in detail.activity"
            :key="event.id"
            class="fd-detail-activity__row"
          >
            <span class="fd-detail-activity__dot" aria-hidden="true" />
            <span class="fd-detail-activity__text">
              <span class="fd-detail-activity__author">
                {{ t('issues.activityCommentAuthor') }}
              </span>
              {{ activityText(event, t, te) }}
            </span>
            <span
              class="fd-detail-activity__time"
              :title="formatDateTime(event.createdAt, app.resolvedLocale)"
            >
              {{ formatRelative(event.createdAt, app.resolvedLocale) }}
            </span>
          </div>
        </div>
        <p v-else class="fd-detail-empty">{{ t('issues.noActivity') }}</p>
      </section>

      <section class="fd-detail-section">
        <h3 class="fd-detail-section__title">
          {{ t('issues.comments') }}
          <span v-if="detail.comments.length" class="fd-detail-count">
            {{ detail.comments.length }}
          </span>
        </h3>
        <div v-if="detail.comments.length" class="fd-detail-comments">
          <div
            v-for="comment in detail.comments"
            :key="comment.id"
            class="fd-detail-comment"
          >
            <div class="fd-detail-comment__meta">
              <MessageSquare :size="12" aria-hidden="true" />
              <span>{{ t('issues.activityCommentAuthor') }}</span>
              <span
                class="fd-detail-comment__time"
                :title="formatDateTime(comment.createdAt, app.resolvedLocale)"
              >
                {{ formatRelative(comment.createdAt, app.resolvedLocale) }}
              </span>
            </div>
            <p class="fd-detail-comment__body">{{ comment.body }}</p>
          </div>
        </div>
        <p v-else class="fd-detail-empty">{{ t('issues.commentsEmpty') }}</p>

        <div class="fd-detail-comment-editor">
          <FdTextarea
            v-model="newComment"
            :rows="3"
            :placeholder="t('issues.commentPlaceholder')"
            @keydown.ctrl.enter.prevent="submitComment"
            @keydown.meta.enter.prevent="submitComment"
          />
          <div class="fd-detail-comment-editor__actions">
            <FdButton
              variant="primary"
              size="sm"
              :disabled="!newComment.trim()"
              :loading="commentBusy"
              @click="submitComment"
            >
              {{ t('issues.addComment') }}
            </FdButton>
          </div>
        </div>
      </section>
    </div>

    <FdConfirmDialog
      v-model="archiveOpen"
      :title="t('issues.archiveTitle')"
      :body="t('issues.archiveBody', { key: issue.key })"
      :confirm-label="t('issues.archiveConfirm')"
      destructive
      :busy="archiveBusy"
      @confirm="confirmArchive"
    />
  </EntityDetailPane>
</template>

<style scoped>
.fd-detail-header {
  display: flex;
  flex: none;
  align-items: center;
  justify-content: space-between;
  gap: var(--fd-space-2);
  min-height: 44px;
  padding: 6px var(--fd-space-4);
  border-bottom: 1px solid var(--fd-border-subtle);
}

.fd-detail-header__identity {
  display: flex;
  align-items: center;
  gap: 6px;
  min-width: 0;
}

.fd-detail-header__key {
  color: var(--fd-text-secondary);
  font-size: var(--fd-text-11);
}

.fd-detail-header__archived {
  color: var(--fd-warning);
  font-size: var(--fd-text-11);
}

.fd-detail-header__actions {
  display: flex;
  flex: none;
  align-items: center;
  gap: 1px;
}

.fd-detail-body {
  flex: 1 1 auto;
  min-height: 0;
  padding: var(--fd-space-4);
  overflow-y: auto;
}

.fd-detail-title {
  width: 100%;
  padding: 2px 0;
  border: 1px solid transparent;
  border-radius: var(--fd-radius-control);
  background: transparent;
  color: var(--fd-text);
  font-size: var(--fd-text-18);
  font-weight: var(--fd-weight-semibold);
  line-height: var(--fd-line-tight);
}

.fd-detail-title:hover {
  border-color: var(--fd-border-subtle);
}

.fd-detail-title:focus {
  outline: none;
  border-color: var(--fd-accent);
  box-shadow: 0 0 0 3px var(--fd-focus-ring);
}

.fd-detail-subtitle {
  margin-top: 4px;
  color: var(--fd-text-muted);
  font-size: var(--fd-text-11);
}

.fd-detail-section {
  padding-top: var(--fd-space-4);
  margin-top: var(--fd-space-4);
  border-top: 1px solid var(--fd-border-subtle);
}

.fd-detail-section:first-of-type {
  margin-top: var(--fd-space-3);
  border-top: 0;
  padding-top: 0;
}

.fd-detail-section__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--fd-space-2);
}

.fd-detail-section__title {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: var(--fd-text-12);
  font-weight: var(--fd-weight-semibold);
  color: var(--fd-text-secondary);
}

.fd-detail-count {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 16px;
  height: 16px;
  padding: 0 4px;
  border-radius: 999px;
  background: var(--fd-surface-selected);
  color: var(--fd-text-muted);
  font-size: var(--fd-text-11);
}

.fd-detail-description {
  position: relative;
}

.fd-detail-description__text {
  color: var(--fd-text-secondary);
  font-size: var(--fd-text-13);
  line-height: var(--fd-line-body);
  white-space: pre-wrap;
}

.fd-detail-description__empty {
  color: var(--fd-text-muted);
  font-size: var(--fd-text-12);
}

.fd-detail-description__edit {
  margin-top: var(--fd-space-2);
  padding: 2px 4px;
  border: 0;
  border-radius: 4px;
  background: transparent;
  color: var(--fd-accent);
  font-size: var(--fd-text-11);
}

.fd-detail-description__edit:hover {
  background: var(--fd-accent-soft);
}

.fd-detail-description-editor {
  display: flex;
  flex-direction: column;
  gap: var(--fd-space-2);
}

.fd-detail-description-editor__actions {
  display: flex;
  justify-content: flex-end;
  gap: var(--fd-space-2);
}

.fd-property-grid {
  display: grid;
  grid-template-columns: 86px minmax(0, 1fr);
  gap: 8px 10px;
  align-items: center;
  margin-top: var(--fd-space-3);
}

.fd-property-grid__label {
  color: var(--fd-text-muted);
  font-size: var(--fd-text-11);
}

.fd-property-grid :deep(.fd-select),
.fd-property-grid :deep(.fd-input) {
  width: 100%;
  max-width: 260px;
}

.fd-detail-labels {
  display: grid;
  grid-template-columns: 86px minmax(0, 1fr);
  gap: 8px 10px;
  align-items: start;
  margin-top: var(--fd-space-3);
}

.fd-detail-labels__items {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
  align-items: center;
}

.fd-detail-labels__item {
  display: inline-flex;
  align-items: center;
  gap: 2px;
}

.fd-detail-labels__remove {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 18px;
  height: 18px;
  padding: 0;
  border: 0;
  border-radius: 4px;
  background: transparent;
  color: var(--fd-text-muted);
}

.fd-detail-labels__remove:hover {
  background: var(--fd-surface-hover);
  color: var(--fd-text);
}

.fd-detail-labels__input {
  width: 180px;
}

.fd-detail-list {
  display: flex;
  flex-direction: column;
  gap: 1px;
  margin-top: var(--fd-space-2);
}

.fd-detail-issue-link {
  display: flex;
  align-items: center;
  gap: 6px;
  width: 100%;
  min-height: 28px;
  padding: 3px 6px;
  overflow: hidden;
  border: 0;
  border-radius: 4px;
  background: transparent;
  color: var(--fd-text-secondary);
  font-size: var(--fd-text-12);
  text-align: left;
  text-decoration: none;
}

.fd-detail-issue-link:hover {
  background: var(--fd-surface-hover);
  color: var(--fd-text);
}

.fd-detail-relation-row {
  display: flex;
  align-items: center;
  gap: 4px;
}

.fd-detail-relation-row .fd-detail-issue-link {
  flex: 1;
}

.fd-detail-relation-kind {
  flex: none;
  min-width: 78px;
  color: var(--fd-text-muted);
  font-size: var(--fd-text-11);
}

.fd-detail-add-relation {
  display: flex;
  gap: 4px;
  align-items: center;
  margin-top: var(--fd-space-2);
}

.fd-detail-add-relation__kind {
  flex: 0 0 104px;
}

.fd-detail-add-relation__issue {
  flex: 1;
  min-width: 0;
}

.fd-detail-empty {
  margin-top: var(--fd-space-2);
  color: var(--fd-text-muted);
  font-size: var(--fd-text-11);
}

.fd-detail-activity {
  display: flex;
  flex-direction: column;
  gap: 1px;
  margin-top: var(--fd-space-2);
}

.fd-detail-activity__row {
  display: flex;
  align-items: baseline;
  gap: 6px;
  min-height: 26px;
  padding: 3px 4px;
  color: var(--fd-text-muted);
  font-size: var(--fd-text-11);
}

.fd-detail-activity__dot {
  flex: none;
  width: 5px;
  height: 5px;
  margin-top: 4px;
  border-radius: 50%;
  background: var(--fd-border);
}

.fd-detail-activity__text {
  flex: 1;
  color: var(--fd-text-secondary);
  line-height: var(--fd-line-body);
}

.fd-detail-activity__author {
  color: var(--fd-text);
  font-weight: var(--fd-weight-medium);
}

.fd-detail-activity__time {
  flex: none;
  color: var(--fd-text-muted);
  white-space: nowrap;
}

.fd-detail-comments {
  display: flex;
  flex-direction: column;
  gap: var(--fd-space-2);
  margin-top: var(--fd-space-2);
}

.fd-detail-comment {
  padding: 8px 10px;
  border: 1px solid var(--fd-border-subtle);
  border-radius: var(--fd-radius-control);
  background: var(--fd-surface-raised);
}

.fd-detail-comment__meta {
  display: flex;
  align-items: center;
  gap: 5px;
  color: var(--fd-text-muted);
  font-size: var(--fd-text-11);
}

.fd-detail-comment__time {
  margin-left: auto;
}

.fd-detail-comment__body {
  margin-top: 4px;
  color: var(--fd-text-secondary);
  font-size: var(--fd-text-12);
  line-height: var(--fd-line-body);
  white-space: pre-wrap;
}

.fd-detail-comment-editor {
  display: flex;
  flex-direction: column;
  gap: var(--fd-space-2);
  margin-top: var(--fd-space-3);
}

.fd-detail-comment-editor__actions {
  display: flex;
  justify-content: flex-end;
}
</style>
