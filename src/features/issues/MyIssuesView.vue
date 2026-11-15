<script setup lang="ts">
import { CircleDot, Plus, X } from '@lucide/vue'
import { computed, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'

import PageHeader from '@/components/layout/PageHeader.vue'
import PageToolbar from '@/components/layout/PageToolbar.vue'
import CreateIssueDialog from '@/components/issues/CreateIssueDialog.vue'
import EntityDetailPane from '@/components/issues/EntityDetailPane.vue'
import IssueBoard from '@/components/issues/IssueBoard.vue'
import IssueDetailPane from '@/components/issues/IssueDetailPane.vue'
import IssueList from '@/components/issues/IssueList.vue'
import FdButton from '@/components/ui/FdButton.vue'
import FdConfirmDialog from '@/components/ui/FdConfirmDialog.vue'
import FdEmptyState from '@/components/ui/FdEmptyState.vue'
import FdErrorState from '@/components/ui/FdErrorState.vue'
import FdIconButton from '@/components/ui/FdIconButton.vue'
import FdInput from '@/components/ui/FdInput.vue'
import FdLoadingSkeleton from '@/components/ui/FdLoadingSkeleton.vue'
import FdSegmented from '@/components/ui/FdSegmented.vue'
import FdSelect from '@/components/ui/FdSelect.vue'
import { statusLabel } from '@/lib/issue'
import { useAppStore } from '@/stores/app'
import { useIssueStore } from '@/stores/issues'
import { useUiStore } from '@/stores/ui'
import type { Priority } from '@/types/ipc'

const issueStore = useIssueStore()
const app = useAppStore()
const ui = useUiStore()
const { t, te } = useI18n()

const createOpen = ref(false)
const createParentId = ref<string | null>(null)
const createParentKey = ref<string | null>(null)
const selectedIds = ref<string[]>([])
const bulkStatus = ref('')
const bulkPriority = ref('')
const bulkBusy = ref(false)
const bulkArchiveOpen = ref(false)

watch(
  () => app.workspace?.id,
  (workspaceId) => {
    if (workspaceId) {
      void issueStore.load()
    }
  },
  { immediate: true },
)

const viewOptions = computed(() => [
  { value: 'list', label: t('pages.myIssues.list') },
  { value: 'board', label: t('pages.myIssues.board') },
])

const searchModel = computed({
  get: () => issueStore.search,
  set: (value: string) => issueStore.setSearch(value),
})

const compact = computed(
  () => issueStore.selectedId !== null && !ui.isNarrowViewport,
)

const selectedCount = computed(() => selectedIds.value.length)
const allSelected = computed(
  () =>
    issueStore.issues.length > 0 &&
    issueStore.issues.every((issue) => selectedIds.value.includes(issue.id)),
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
    label: t(`priority.${value}`),
  })),
)

function setView(value: string): void {
  issueStore.setView(value === 'board' ? 'board' : 'list')
  selectedIds.value = []
}

function openCreate(): void {
  createParentId.value = null
  createParentKey.value = null
  createOpen.value = true
}

function openSubIssue(parentId: string, parentKey: string): void {
  createParentId.value = parentId
  createParentKey.value = parentKey
  createOpen.value = true
}

function toggle(id: string): void {
  selectedIds.value = selectedIds.value.includes(id)
    ? selectedIds.value.filter((value) => value !== id)
    : [...selectedIds.value, id]
}

function toggleAll(): void {
  if (allSelected.value) {
    selectedIds.value = []
  } else {
    selectedIds.value = issueStore.issues.map((issue) => issue.id)
  }
}

async function applyBulkStatus(value: string): Promise<void> {
  if (!value || selectedIds.value.length === 0) {
    bulkStatus.value = ''
    return
  }
  bulkBusy.value = true
  const ok = await issueStore.batchUpdate(selectedIds.value, { statusId: value })
  bulkBusy.value = false
  bulkStatus.value = ''
  if (ok) {
    selectedIds.value = []
  }
}

async function applyBulkPriority(value: string): Promise<void> {
  if (!value || selectedIds.value.length === 0) {
    bulkPriority.value = ''
    return
  }
  bulkBusy.value = true
  const ok = await issueStore.batchUpdate(selectedIds.value, {
    priority: value as Priority,
  })
  bulkBusy.value = false
  bulkPriority.value = ''
  if (ok) {
    selectedIds.value = []
  }
}

async function archiveSelected(): Promise<void> {
  if (selectedIds.value.length === 0 || bulkBusy.value) {
    return
  }
  bulkBusy.value = true
  const ok = await issueStore.batchUpdate(selectedIds.value, { archive: true })
  bulkBusy.value = false
  if (ok) {
    bulkArchiveOpen.value = false
    selectedIds.value = []
  }
}
</script>

<template>
  <div class="fd-issues-page" data-testid="page-content">
    <PageHeader
      :title="t('pages.myIssues.title')"
      :description="t('pages.myIssues.description')"
    >
      <template #actions>
        <FdButton variant="primary" @click="openCreate">
          <Plus :size="14" aria-hidden="true" />
          {{ t('issues.newIssue') }}
        </FdButton>
      </template>
    </PageHeader>

    <PageToolbar>
      <template #left>
        <FdSegmented
          :model-value="issueStore.view"
          :options="viewOptions"
          :label="t('pages.myIssues.list')"
          @update:model-value="setView"
        />
        <FdInput
          v-model="searchModel"
          type="search"
          class="fd-issues-search"
          :placeholder="t('issues.searchPlaceholder')"
          :aria-label="t('issues.searchPlaceholder')"
        />
      </template>
      <template #right>
        <span v-tooltip="t('pages.myIssues.toolbarUnavailable')">
          <FdButton variant="ghost" size="sm" disabled>
            {{ t('pages.myIssues.filter') }}
          </FdButton>
        </span>
        <span v-tooltip="t('pages.myIssues.toolbarUnavailable')">
          <FdButton variant="ghost" size="sm" disabled>
            {{ t('pages.myIssues.sort') }}
          </FdButton>
        </span>
        <span v-tooltip="t('pages.myIssues.toolbarUnavailable')">
          <FdButton variant="ghost" size="sm" disabled>
            {{ t('pages.myIssues.savedView') }}
          </FdButton>
        </span>
      </template>
    </PageToolbar>

    <div v-if="selectedCount > 0" class="fd-bulk-bar">
      <span class="fd-bulk-bar__count">
        {{ t('issues.bulkSelected', { count: selectedCount }) }}
      </span>
      <FdSelect
        :model-value="bulkStatus"
        class="fd-bulk-bar__select"
        :aria-label="t('issues.bulkStatus')"
        :disabled="bulkBusy"
        @update:model-value="applyBulkStatus"
      >
        <option value="">{{ t('issues.bulkStatus') }}</option>
        <option
          v-for="option in statusOptions"
          :key="option.value"
          :value="option.value"
        >
          {{ option.label }}
        </option>
      </FdSelect>
      <FdSelect
        :model-value="bulkPriority"
        class="fd-bulk-bar__select"
        :aria-label="t('issues.bulkPriority')"
        :disabled="bulkBusy"
        @update:model-value="applyBulkPriority"
      >
        <option value="">{{ t('issues.bulkPriority') }}</option>
        <option
          v-for="option in priorityOptions"
          :key="option.value"
          :value="option.value"
        >
          {{ option.label }}
        </option>
      </FdSelect>
      <FdButton
        variant="destructive"
        size="sm"
        :disabled="bulkBusy"
        @click="bulkArchiveOpen = true"
      >
        {{ t('issues.bulkArchive') }}
      </FdButton>
      <FdButton
        variant="ghost"
        size="sm"
        :disabled="bulkBusy"
        @click="selectedIds = []"
      >
        {{ t('issues.bulkClear') }}
      </FdButton>
    </div>

    <div class="fd-issues-content">
      <div class="fd-issues-main">
        <div class="fd-issues-scroll">
          <div v-if="issueStore.loading" class="fd-issues-state">
            <FdLoadingSkeleton :rows="8" :height="40" />
          </div>

          <div v-else-if="issueStore.error" class="fd-issues-state">
            <FdErrorState
              :title="t('issues.loadError')"
              :message="issueStore.error.message"
              :code="issueStore.error.code"
              :detail="issueStore.error.detail"
              :retry-label="t('issues.retry')"
              @retry="issueStore.load()"
            />
          </div>

          <div v-else-if="issueStore.isEmpty" class="fd-issues-state">
            <FdEmptyState
              :icon="CircleDot"
              :title="t('pages.myIssues.emptyTitle')"
              :body="t('pages.myIssues.emptyBody')"
            >
              <FdButton variant="primary" @click="openCreate">
                <Plus :size="14" aria-hidden="true" />
                {{ t('issues.createFirst') }}
              </FdButton>
            </FdEmptyState>
          </div>

          <IssueList
            v-else-if="issueStore.view === 'list'"
            :issues="issueStore.issues"
            :selected-ids="selectedIds"
            :compact="compact"
            @select="issueStore.selectIssue"
            @toggle="toggle"
            @toggle-all="toggleAll"
          />

          <IssueBoard
            v-else
            :columns="issueStore.board"
            @select="issueStore.selectIssue"
          />
        </div>
      </div>

      <EntityDetailPane
        v-if="issueStore.selectedId && !issueStore.detail"
        :label="t('issues.detailLoading')"
        @close="issueStore.closeDetail()"
      >
        <header class="fd-detail-loading-header">
          <span>{{ t('issues.detailLoading') }}</span>
          <FdIconButton
            :label="t('issues.close')"
            @click="issueStore.closeDetail()"
          >
            <X :size="16" />
          </FdIconButton>
        </header>
        <div class="fd-detail-loading-body">
          <FdLoadingSkeleton :rows="6" :height="40" />
        </div>
      </EntityDetailPane>

      <IssueDetailPane
        v-else-if="issueStore.selectedId"
        @create-subissue="openSubIssue"
      />
    </div>

    <CreateIssueDialog
      v-model="createOpen"
      :parent-id="createParentId"
      :parent-label="createParentKey"
    />

    <FdConfirmDialog
      v-model="bulkArchiveOpen"
      :title="t('issues.bulkArchiveTitle')"
      :body="t('issues.bulkArchiveBody', { count: selectedCount })"
      :confirm-label="t('issues.bulkArchiveConfirm')"
      destructive
      :busy="bulkBusy"
      @confirm="archiveSelected"
    />
  </div>
</template>

<style scoped>
.fd-issues-page {
  display: flex;
  flex-direction: column;
  width: 100%;
  height: 100%;
  min-width: 0;
  min-height: 0;
  overflow: hidden;
}

.fd-issues-search {
  width: 220px;
}

.fd-bulk-bar {
  display: flex;
  flex: none;
  align-items: center;
  gap: var(--fd-space-2);
  height: 40px;
  padding: 0 var(--fd-content-pad);
  border-bottom: 1px solid var(--fd-border-subtle);
  background: var(--fd-surface-selected);
}

.fd-bulk-bar__count {
  color: var(--fd-text-secondary);
  font-size: var(--fd-text-12);
  font-weight: var(--fd-weight-medium);
}

.fd-bulk-bar__select {
  width: 150px;
}

.fd-issues-content {
  position: relative;
  display: flex;
  flex: 1;
  min-width: 0;
  min-height: 0;
}

.fd-issues-main {
  display: flex;
  flex: 1;
  flex-direction: column;
  min-width: 0;
  min-height: 0;
  overflow: hidden;
}

.fd-issues-scroll {
  flex: 1;
  min-width: 0;
  min-height: 0;
  overflow-y: auto;
  overflow-x: hidden;
}

.fd-issues-state {
  display: flex;
  justify-content: center;
  padding: var(--fd-space-6) var(--fd-content-pad);
}

.fd-issues-state :deep(.fd-skeleton) {
  max-width: 900px;
}

.fd-detail-loading-header {
  display: flex;
  flex: none;
  align-items: center;
  justify-content: space-between;
  height: 44px;
  padding: 0 var(--fd-space-4);
  border-bottom: 1px solid var(--fd-border-subtle);
  color: var(--fd-text-muted);
  font-size: var(--fd-text-12);
}

.fd-detail-loading-body {
  padding: var(--fd-space-4);
}

@media (max-width: 1199px) {
  .fd-issues-search {
    width: 170px;
  }
}
</style>
