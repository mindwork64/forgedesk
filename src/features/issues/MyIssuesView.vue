<script setup lang="ts">
import { CircleDot, Plus, X } from '@lucide/vue'
import { computed, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRoute, useRouter } from 'vue-router'

import PageHeader from '@/components/layout/PageHeader.vue'
import PageToolbar from '@/components/layout/PageToolbar.vue'
import ActiveFilterChips from '@/components/issues/ActiveFilterChips.vue'
import CreateIssueDialog from '@/components/issues/CreateIssueDialog.vue'
import EntityDetailPane from '@/components/issues/EntityDetailPane.vue'
import IssueBoard from '@/components/issues/IssueBoard.vue'
import IssueDetailPane from '@/components/issues/IssueDetailPane.vue'
import IssueList from '@/components/issues/IssueList.vue'
import FilterBuilder from '@/components/issues/FilterBuilder.vue'
import GroupMenu from '@/components/issues/GroupMenu.vue'
import SavedViewsMenu from '@/components/issues/SavedViewsMenu.vue'
import SaveViewDialog from '@/components/issues/SaveViewDialog.vue'
import SortMenu from '@/components/issues/SortMenu.vue'
import FdButton from '@/components/ui/FdButton.vue'
import FdConfirmDialog from '@/components/ui/FdConfirmDialog.vue'
import FdEmptyState from '@/components/ui/FdEmptyState.vue'
import FdErrorState from '@/components/ui/FdErrorState.vue'
import FdIconButton from '@/components/ui/FdIconButton.vue'
import FdInput from '@/components/ui/FdInput.vue'
import FdLoadingSkeleton from '@/components/ui/FdLoadingSkeleton.vue'
import FdPopover from '@/components/ui/FdPopover.vue'
import FdSegmented from '@/components/ui/FdSegmented.vue'
import FdSelect from '@/components/ui/FdSelect.vue'
import { countFilterConditions, type FilterMetadata } from '@/lib/filter'
import { statusLabel } from '@/lib/issue'
import { useAppStore } from '@/stores/app'
import { useIssueStore } from '@/stores/issues'
import { useUiStore } from '@/stores/ui'
import type { FilterNode, Priority, SavedView, SortSpec } from '@/types/ipc'

const issueStore = useIssueStore()
const app = useAppStore()
const route = useRoute()
const router = useRouter()
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
const filterOpen = ref(false)
const sortOpen = ref(false)
const groupOpen = ref(false)
const viewsOpen = ref(false)
const saveViewOpen = ref(false)
const deleteViewTarget = ref<SavedView | null>(null)
const deleteViewOpen = ref(false)

watch(
  [() => app.workspace?.id, () => route.query.view],
  async ([workspaceId, viewId]) => {
    if (!workspaceId) {
      return
    }
    if (issueStore.savedViews.length === 0) {
      await issueStore.loadSavedViews()
    }
    if (typeof viewId === 'string') {
      const view = issueStore.savedViews.find((item) => item.id === viewId)
      if (view) {
        issueStore.applyView(view)
        return
      }
    }
    await issueStore.load()
  },
  { immediate: true },
)

watch(
  () => route.query.issue,
  (issueId) => {
    if (typeof issueId === 'string') {
      void issueStore.selectIssue(issueId)
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

const filterMetadata = computed<FilterMetadata>(() => ({
  statuses: issueStore.statuses,
  labels: issueStore.labels,
  projects: issueStore.projects,
  issues: issueStore.issues,
}))

const activeFilter = computed<FilterNode | null>(
  () => issueStore.filter ?? null,
)

const activeFilterCount = computed(() =>
  countFilterConditions(activeFilter.value),
)

const sortLabel = computed(() => {
  const current = issueStore.sort[0]
  if (!current) {
    return t('pages.myIssues.sort')
  }
  const key =
    current.field === 'title'
      ? 'sortMenu.titleField'
      : `sortMenu.${current.field}`
  return `${t('pages.myIssues.sort')} · ${te(key) ? t(key) : current.field}`
})

const savedViewLabel = computed(
  () => issueStore.activeView?.name ?? t('pages.myIssues.savedView'),
)

const groupLabel = computed(() => {
  if (!issueStore.groupBy) {
    return t('groupMenu.title')
  }
  const key = `filters.fields.${issueStore.groupBy}`
  return te(key)
    ? `${t('groupMenu.title')} · ${t(key)}`
    : t('groupMenu.title')
})

const saveViewName = computed(() => issueStore.activeView?.name ?? '')
const saveViewOverwriteId = computed(() => issueStore.activeView?.id ?? null)

const filterButtonLabel = computed(() =>
  activeFilterCount.value > 0
    ? `${t('pages.myIssues.filter')} · ${activeFilterCount.value}`
    : t('pages.myIssues.filter'),
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

function clearViewQuery(): void {
  if (!route.query.view) {
    return
  }
  const query = { ...route.query }
  delete query.view
  void router.replace({ path: '/my-issues', query })
}

function applyFilter(next: FilterNode | null): void {
  issueStore.setFilter(next)
  clearViewQuery()
  filterOpen.value = false
}

function removeFilter(index: number): void {
  const current = issueStore.filter
  if (!current) {
    return
  }
  if (current.kind === 'condition') {
    issueStore.setFilter(null)
    return
  }
  const conditions = current.conditions.filter((_, idx) => idx !== index)
  issueStore.setFilter(
    conditions.length ? { ...current, conditions } : null,
  )
  clearViewQuery()
}

function clearFilter(): void {
  issueStore.setFilter(null)
  clearViewQuery()
  filterOpen.value = false
}

function applySort(next: SortSpec[]): void {
  issueStore.setSort(next)
  clearViewQuery()
  sortOpen.value = false
}

function applyGroup(next: string | null): void {
  issueStore.setGroupBy(next)
  clearViewQuery()
  groupOpen.value = false
}

function applySavedView(view: SavedView): void {
  issueStore.applyView(view)
  viewsOpen.value = false
  void router.replace({ path: '/my-issues', query: { view: view.id } })
}

function openSaveView(): void {
  viewsOpen.value = false
  saveViewOpen.value = true
}

async function saveView(name: string, overwriteId: string | null): Promise<void> {
  const saved = await issueStore.saveView(name, overwriteId)
  if (!saved) {
    return
  }
  saveViewOpen.value = false
  await router.replace({ path: '/my-issues', query: { view: saved.id } })
}

async function togglePinnedView(view: SavedView): Promise<void> {
  await issueStore.togglePin(view)
}

function requestDeleteView(view: SavedView): void {
  viewsOpen.value = false
  deleteViewTarget.value = view
  deleteViewOpen.value = true
}

async function confirmDeleteView(): Promise<void> {
  if (!deleteViewTarget.value) {
    return
  }
  const id = deleteViewTarget.value.id
  const ok = await issueStore.deleteView(id)
  if (ok) {
    deleteViewOpen.value = false
    deleteViewTarget.value = null
    if (route.query.view === id) {
      await router.replace({ path: '/my-issues' })
    }
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
        <FdPopover
          v-model:open="filterOpen"
          :width="620"
          align="end"
          :aria-label="t('filters.title')"
        >
          <template #trigger="{ toggle }">
            <FdButton
              :variant="activeFilterCount ? 'secondary' : 'ghost'"
              size="sm"
              @click="toggle"
            >
              {{ filterButtonLabel }}
            </FdButton>
          </template>
          <FilterBuilder
            :initial="activeFilter"
            :metadata="filterMetadata"
            @apply="applyFilter"
            @clear="clearFilter"
          />
        </FdPopover>

        <FdPopover
          v-model:open="sortOpen"
          :width="300"
          align="end"
          :aria-label="t('sortMenu.title')"
        >
          <template #trigger="{ toggle }">
            <FdButton variant="ghost" size="sm" @click="toggle">
              {{ sortLabel }}
            </FdButton>
          </template>
          <SortMenu :current="issueStore.sort" @apply="applySort" />
        </FdPopover>

        <FdPopover
          v-model:open="groupOpen"
          :width="260"
          align="end"
          :aria-label="t('groupMenu.title')"
        >
          <template #trigger="{ toggle }">
            <FdButton variant="ghost" size="sm" @click="toggle">
              {{ groupLabel }}
            </FdButton>
          </template>
          <GroupMenu :current="issueStore.groupBy" @apply="applyGroup" />
        </FdPopover>

        <FdPopover
          v-model:open="viewsOpen"
          :width="320"
          align="end"
          :aria-label="t('views.savedView')"
        >
          <template #trigger="{ toggle }">
            <FdButton variant="ghost" size="sm" @click="toggle">
              {{ savedViewLabel }}
            </FdButton>
          </template>
          <SavedViewsMenu
            :views="issueStore.savedViews"
            :active-id="issueStore.activeViewId"
            @apply="applySavedView"
            @pin="togglePinnedView"
            @delete="requestDeleteView"
            @save="openSaveView"
          />
        </FdPopover>
      </template>
    </PageToolbar>

    <ActiveFilterChips
      v-if="activeFilter"
      :filter="activeFilter"
      :metadata="filterMetadata"
      @remove="removeFilter"
      @clear="clearFilter"
    />

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
            :group-by="issueStore.groupBy"
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

    <SaveViewDialog
      v-model:open="saveViewOpen"
      :initial-name="saveViewName"
      :overwrite-id="saveViewOverwriteId"
      @save="saveView"
    />

    <FdConfirmDialog
      v-model="deleteViewOpen"
      :title="t('views.deleteTitle')"
      :body="t('views.deleteBody', { name: deleteViewTarget?.name ?? '' })"
      :confirm-label="t('views.deleteConfirm')"
      destructive
      @confirm="confirmDeleteView"
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
  display: flex;
  flex: 1;
  flex-direction: column;
  min-width: 0;
  min-height: 0;
  overflow: hidden;
}

.fd-issues-state {
  display: flex;
  justify-content: center;
  max-height: 100%;
  padding: var(--fd-space-6) var(--fd-content-pad);
  overflow-y: auto;
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
