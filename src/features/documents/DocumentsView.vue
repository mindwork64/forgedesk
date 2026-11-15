<script setup lang="ts">
import { FileText, Plus, Trash2 } from '@lucide/vue'
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRoute, useRouter } from 'vue-router'

import PageHeader from '@/components/layout/PageHeader.vue'
import FdButton from '@/components/ui/FdButton.vue'
import FdConfirmDialog from '@/components/ui/FdConfirmDialog.vue'
import FdEmptyState from '@/components/ui/FdEmptyState.vue'
import FdErrorState from '@/components/ui/FdErrorState.vue'
import FdInput from '@/components/ui/FdInput.vue'
import FdLoadingSkeleton from '@/components/ui/FdLoadingSkeleton.vue'
import FdSelect from '@/components/ui/FdSelect.vue'
import FdTextarea from '@/components/ui/FdTextarea.vue'
import { copyText } from '@/lib/clipboard'
import { formatRelative } from '@/lib/format'
import { useAppStore } from '@/stores/app'
import { usePlanningStore } from '@/stores/planning'
import { useToastStore } from '@/stores/toasts'
import type { Document } from '@/types/ipc'

const planning = usePlanningStore()
const app = useAppStore()
const toast = useToastStore()
const route = useRoute()
const router = useRouter()
const { t } = useI18n()

const search = ref('')
const title = ref('')
const body = ref('')
const projectId = ref('')
const initiativeId = ref('')
const saveState = ref<'saved' | 'saving' | 'dirty' | 'error'>('saved')
const deleteOpen = ref(false)
const hydrating = ref(false)
let saveTimer: ReturnType<typeof setTimeout> | null = null

const filtered = computed(() => {
  const needle = search.value.trim().toLowerCase()
  if (!needle) {
    return planning.documents
  }
  return planning.documents.filter(
    (document) =>
      document.title.toLowerCase().includes(needle) ||
      document.body.toLowerCase().includes(needle),
  )
})
const selected = computed(() => planning.document)

function hydrate(document: Document | null): void {
  hydrating.value = true
  title.value = document?.title ?? ''
  body.value = document?.body ?? ''
  projectId.value = document?.projectId ?? ''
  initiativeId.value = document?.initiativeId ?? ''
  saveState.value = 'saved'
  void nextTick(() => {
    hydrating.value = false
  })
}

async function selectDocument(document: Document): Promise<void> {
  await planning.selectDocument(document.id)
  hydrate(planning.document)
  if (route.query.doc !== document.id) {
    await router.replace({ path: '/documents', query: { doc: document.id } })
  }
}

async function load(): Promise<void> {
  await Promise.all([
    planning.loadDocuments(),
    planning.loadProjects(),
    planning.loadInitiatives(),
  ])
  const requested =
    typeof route.query.doc === 'string' ? route.query.doc : ''
  const target =
    planning.documents.find((document) => document.id === requested) ??
    planning.documents[0] ??
    null
  if (target) {
    await selectDocument(target)
  } else {
    hydrate(null)
  }
}

async function createDocument(): Promise<void> {
  const created = await planning.createDocument({
    title: t('documents.untitled'),
    body: '',
  })
  if (created) {
    await selectDocument(created)
  }
}

async function save(): Promise<void> {
  if (!selected.value) {
    return
  }
  saveState.value = 'saving'
  const updated = await planning.updateDocument(selected.value.id, {
    title: title.value.trim() || t('documents.untitled'),
    body: body.value,
    projectId: projectId.value || null,
    initiativeId: initiativeId.value || null,
  })
  saveState.value = updated ? 'saved' : 'error'
}

function scheduleSave(): void {
  if (hydrating.value || !selected.value) {
    return
  }
  saveState.value = 'dirty'
  if (saveTimer) {
    clearTimeout(saveTimer)
  }
  saveTimer = setTimeout(() => {
    void save()
  }, 700)
}

watch([title, body, projectId, initiativeId], scheduleSave)

async function exportMarkdown(): Promise<void> {
  if (!selected.value) {
    return
  }
  const markdown = await planning.exportDocument(selected.value.id)
  if (!markdown) {
    return
  }
  const copied = await copyText(markdown)
  if (copied) {
    toast.success(t('documents.exported'))
  } else {
    toast.error(t('documents.exportError'))
  }
}

async function deleteDocument(): Promise<void> {
  if (!selected.value) {
    return
  }
  const ok = await planning.deleteDocument(selected.value.id)
  if (!ok) {
    return
  }
  deleteOpen.value = false
  await router.replace({ path: '/documents' })
  await load()
}

onMounted(load)

watch(
  () => route.query.doc,
  async (id) => {
    if (typeof id !== 'string' || id === selected.value?.id) {
      return
    }
    const document = planning.documents.find((item) => item.id === id)
    if (document) {
      await planning.selectDocument(id)
      hydrate(planning.document)
    }
  },
)

onBeforeUnmount(() => {
  if (saveTimer) {
    clearTimeout(saveTimer)
    if (saveState.value === 'dirty') {
      void save()
    }
  }
})
</script>

<template>
  <div class="fd-documents-page">
    <PageHeader
      :title="t('pages.documents.title')"
      :description="t('pages.documents.description')"
    >
      <template #actions>
        <FdButton variant="primary" @click="createDocument">
          <Plus :size="14" aria-hidden="true" />
          {{ t('documents.newDocument') }}
        </FdButton>
      </template>
    </PageHeader>

    <div class="fd-documents-layout">
      <aside class="fd-documents-list">
        <div class="fd-documents-search">
          <FdInput
            v-model="search"
            type="search"
            :placeholder="t('documents.searchPlaceholder')"
            :aria-label="t('documents.searchPlaceholder')"
          />
        </div>
        <div v-if="planning.loading" class="fd-documents-state">
          <FdLoadingSkeleton :rows="6" :height="42" />
        </div>
        <div v-else-if="planning.error" class="fd-documents-state">
          <FdErrorState
            :title="t('documents.createError')"
            :message="planning.error.message"
            :code="planning.error.code"
            :detail="planning.error.detail"
            :retry-label="t('common.retry')"
            @retry="load"
          />
        </div>
        <div v-else-if="filtered.length === 0" class="fd-documents-state">
          <FdEmptyState
            :icon="FileText"
            :title="t('documents.emptyTitle')"
            :body="t('documents.emptyBody')"
          >
            <FdButton variant="primary" size="sm" @click="createDocument">
              {{ t('documents.createFirst') }}
            </FdButton>
          </FdEmptyState>
        </div>
        <button
          v-for="document in filtered"
          :key="document.id"
          type="button"
          class="fd-documents-item"
          :class="{ 'fd-documents-item--active': document.id === selected?.id }"
          @click="selectDocument(document)"
        >
          <strong>{{ document.title }}</strong>
          <span>{{ formatRelative(document.updatedAt, app.resolvedLocale) }}</span>
        </button>
      </aside>

      <section v-if="selected" class="fd-document-editor">
        <div class="fd-document-editor__head">
          <FdInput
            v-model="title"
            class="fd-document-title"
            :aria-label="t('documents.titlePlaceholder')"
            :placeholder="t('documents.titlePlaceholder')"
          />
          <div class="fd-document-editor__actions">
            <span class="fd-document-save-state" :class="`fd-document-save-state--${saveState}`">
              {{
                saveState === 'saved'
                  ? t('documents.saved')
                  : saveState === 'saving'
                    ? t('documents.saving')
                    : saveState === 'error'
                      ? t('documents.saveError')
                      : t('documents.dirty')
              }}
            </span>
            <FdButton variant="ghost" size="sm" @click="exportMarkdown">
              {{ t('documents.exportMarkdown') }}
            </FdButton>
            <FdButton variant="ghost" size="sm" @click="deleteOpen = true">
              <Trash2 :size="13" aria-hidden="true" />
              {{ t('common.delete') }}
            </FdButton>
          </div>
        </div>

        <div class="fd-document-editor__meta">
          <FdSelect v-model="projectId" :aria-label="t('documents.relatedProject')">
            <option value="">{{ t('documents.noProject') }}</option>
            <option v-for="project in planning.projects" :key="project.id" :value="project.id">
              {{ project.name }}
            </option>
          </FdSelect>
          <FdSelect v-model="initiativeId" :aria-label="t('documents.relatedInitiative')">
            <option value="">{{ t('documents.noInitiative') }}</option>
            <option
              v-for="initiative in planning.initiatives"
              :key="initiative.id"
              :value="initiative.id"
            >
              {{ initiative.name }}
            </option>
          </FdSelect>
        </div>

        <FdTextarea
          v-model="body"
          class="fd-document-body"
          :rows="24"
          :placeholder="t('documents.bodyPlaceholder')"
          :aria-label="t('documents.bodyPlaceholder')"
        />
      </section>

      <section v-else class="fd-document-editor fd-document-editor--empty">
        <FdEmptyState
          :icon="FileText"
          :title="t('documents.emptyTitle')"
          :body="t('documents.emptyBody')"
        >
          <FdButton variant="primary" @click="createDocument">
            {{ t('documents.createFirst') }}
          </FdButton>
        </FdEmptyState>
      </section>
    </div>

    <FdConfirmDialog
      v-model="deleteOpen"
      :title="t('documents.deleteTitle')"
      :body="t('documents.deleteBody')"
      :confirm-label="t('documents.deleteConfirm')"
      destructive
      @confirm="deleteDocument"
    />
  </div>
</template>

<style scoped>
.fd-documents-page {
  display: flex;
  flex-direction: column;
  width: 100%;
  height: 100%;
  min-width: 0;
  min-height: 0;
  overflow: hidden;
}

.fd-documents-layout {
  display: flex;
  flex: 1;
  min-height: 0;
  border-top: 1px solid var(--fd-border-subtle);
}

.fd-documents-list {
  display: flex;
  flex: 0 0 260px;
  flex-direction: column;
  min-height: 0;
  border-right: 1px solid var(--fd-border);
  overflow-y: auto;
}

.fd-documents-search {
  flex: none;
  padding: var(--fd-space-2);
}

.fd-documents-state {
  display: flex;
  justify-content: center;
  padding: var(--fd-space-4);
}

.fd-documents-item {
  display: flex;
  flex-direction: column;
  gap: 2px;
  width: 100%;
  min-height: 46px;
  padding: 7px var(--fd-space-3);
  border: 0;
  border-bottom: 1px solid var(--fd-border-subtle);
  background: transparent;
  color: var(--fd-text-secondary);
  text-align: left;
  cursor: pointer;
}

.fd-documents-item strong {
  overflow: hidden;
  font-size: var(--fd-text-12);
  font-weight: var(--fd-weight-medium);
  text-overflow: ellipsis;
  white-space: nowrap;
}

.fd-documents-item span {
  color: var(--fd-text-muted);
  font-size: var(--fd-text-11);
}

.fd-documents-item:hover,
.fd-documents-item--active {
  background: var(--fd-surface-selected);
}

.fd-documents-item--active strong {
  color: var(--fd-text-primary);
}

.fd-document-editor {
  display: flex;
  flex: 1;
  flex-direction: column;
  min-width: 0;
  min-height: 0;
  padding: var(--fd-space-3) var(--fd-content-pad);
  overflow: hidden;
}

.fd-document-editor--empty {
  justify-content: center;
  align-items: center;
}

.fd-document-editor__head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--fd-space-3);
  flex: none;
}

.fd-document-title {
  max-width: 560px;
  font-size: var(--fd-text-16);
  font-weight: var(--fd-weight-semibold);
}

.fd-document-editor__actions {
  display: flex;
  align-items: center;
  gap: var(--fd-space-2);
}

.fd-document-save-state {
  color: var(--fd-text-muted);
  font-size: var(--fd-text-11);
}

.fd-document-save-state--dirty {
  color: var(--fd-warning);
}

.fd-document-save-state--error {
  color: var(--fd-danger);
}

.fd-document-editor__meta {
  display: flex;
  gap: var(--fd-space-2);
  flex: none;
  padding: var(--fd-space-2) 0;
}

.fd-document-editor__meta :deep(.fd-select) {
  width: 180px;
}

.fd-document-body {
  flex: 1;
  min-height: 0;
  resize: none;
  font-family: var(--fd-font-mono);
  font-size: var(--fd-text-12);
  line-height: 1.55;
}

.fd-document-body :deep(textarea) {
  height: 100%;
}

@media (max-width: 899px) {
  .fd-documents-list {
    flex-basis: 200px;
  }

  .fd-document-editor__head {
    align-items: flex-start;
    flex-direction: column;
  }
}
</style>
