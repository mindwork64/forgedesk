<script setup lang="ts">
import { ArrowLeft, GitBranch, Plus } from '@lucide/vue'
import { computed, onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRoute, useRouter } from 'vue-router'

import PageHeader from '@/components/layout/PageHeader.vue'
import FdButton from '@/components/ui/FdButton.vue'
import FdErrorState from '@/components/ui/FdErrorState.vue'
import FdInput from '@/components/ui/FdInput.vue'
import FdLoadingSkeleton from '@/components/ui/FdLoadingSkeleton.vue'
import { api, toIpcError } from '@/lib/ipc'
import { formatRelative } from '@/lib/format'
import { useAppStore } from '@/stores/app'
import { useGitStore } from '@/stores/git'

type Tab = 'overview' | 'branches' | 'commits'

const route = useRoute()
const router = useRouter()
const git = useGitStore()
const app = useAppStore()
const { t } = useI18n()

const activeTab = ref<Tab>('overview')
const newBranch = ref('')
const attachPath = ref('')
const attaching = ref(false)
const attachError = ref<string | null>(null)
const repositoryId = computed(() => String(route.params.id ?? ''))
const overview = computed(() => git.overview)
const repo = computed(() => git.repository ?? overview.value?.repository ?? null)
const status = computed(() => overview.value?.status ?? null)
const branches = computed(() => overview.value?.branches ?? [])
const commits = computed(() => overview.value?.log ?? [])
const diff = computed(
  () =>
    overview.value?.diff ?? {
      filesChanged: 0,
      insertions: 0,
      deletions: 0,
      summary: '',
    },
)

const tabs = computed(() => [
  { id: 'overview' as const, label: t('repositories.overview') },
  { id: 'branches' as const, label: t('repositories.branches') },
  { id: 'commits' as const, label: t('repositories.commits') },
])

async function attachCheckout(): Promise<void> {
  if (!repo.value || !attachPath.value.trim() || attaching.value) return
  attaching.value = true
  attachError.value = null
  try {
    git.repository = await api.attachRepositoryCheckout(repo.value.id, attachPath.value.trim())
    await git.loadOverview(repo.value.id)
    attachPath.value = ''
  } catch (caught) {
    attachError.value = toIpcError(caught).message
  } finally {
    attaching.value = false
  }
}

async function load(): Promise<void> {
  if (!repositoryId.value) return
  await git.loadRepository(repositoryId.value)
  if (repo.value && repo.value.checkouts.length > 0) {
    await git.loadOverview(repositoryId.value)
  }
}

onMounted(load)
watch(repositoryId, load)

async function createBranch(): Promise<void> {
  if (!newBranch.value.trim()) {
    return
  }
  const ok = await git.createBranch(repositoryId.value, newBranch.value.trim())
  if (ok) {
    newBranch.value = ''
  }
}

async function switchBranch(branch: string): Promise<void> {
  await git.switchBranch(repositoryId.value, branch)
}
</script>

<template>
  <div class="fd-repository-page">
    <div class="fd-repository-back">
      <FdButton variant="ghost" size="sm" @click="router.push('/repositories')">
        <ArrowLeft :size="13" aria-hidden="true" />
        {{ t('pages.repositories.title') }}
      </FdButton>
    </div>

    <div v-if="git.loading && !overview" class="fd-repository-state">
      <FdLoadingSkeleton :rows="8" :height="42" />
    </div>
    <div v-else-if="git.error" class="fd-repository-state">
      <FdErrorState
        :title="t('repositories.addError')"
        :message="git.error.message"
        :code="git.error.code"
        :detail="git.error.detail"
        :retry-label="t('common.retry')"
        @retry="load"
      />
    </div>
    <div v-else-if="repo && repo.checkouts.length === 0" class="fd-repository-state">
      <PageHeader :title="repo.fullName" :description="t('repositories.remoteOnly')" />
      <section class="fd-repository-section">
        <h2>{{ t('repositories.details') }}</h2>
        <dl class="fd-repository-facts">
          <div><dt>{{ t('repositories.columns.provider') }}</dt><dd>{{ repo.provider }}</dd></div>
          <div><dt>{{ t('repositories.columns.branch') }}</dt><dd>{{ repo.defaultBranch ?? '—' }}</dd></div>
          <div><dt>{{ t('repositories.columns.project') }}</dt><dd>{{ repo.projects.map((project) => project.name).join(', ') || '—' }}</dd></div>
          <div><dt>Remote</dt><dd>{{ repo.cloneUrl ?? repo.htmlUrl ?? '—' }}</dd></div>
        </dl>
        <p class="fd-repository-muted">{{ t('repositories.remoteOnlyHint') }}</p>
        <div class="fd-repository-attach">
          <FdInput v-model="attachPath" :placeholder="t('repositories.attachPathPlaceholder')" />
          <FdButton variant="secondary" :loading="attaching" @click="attachCheckout">
            {{ t('repositories.attachCheckout') }}
          </FdButton>
        </div>
        <p v-if="attachError" class="fd-repository-error">{{ attachError }}</p>
      </section>
    </div>

    <template v-else-if="repo && status">
      <PageHeader :title="repo.fullName" :description="repo.checkouts[0]?.path ?? t('repositories.remoteOnly')">
        <template #actions>
          <span class="fd-repository-status" :class="{ 'fd-repository-status--dirty': status.dirty }">
            {{ status.dirty ? t('repositories.dirty') : t('repositories.clean') }}
          </span>
        </template>
      </PageHeader>

      <div class="fd-repository-meta">
        <span>{{ repo.provider === 'local' ? t('repositories.providerLocal') : repo.provider }}</span>
        <span aria-hidden="true">·</span>
        <span><GitBranch :size="12" aria-hidden="true" /> {{ status.branch }}</span>
        <span aria-hidden="true">·</span>
        <span>{{ status.upstream ? t('repositories.upstream') + ': ' + status.upstream : t('repositories.noUpstream') }}</span>
      </div>

      <nav class="fd-repository-tabs" :aria-label="repo.name">
        <button
          v-for="tab in tabs"
          :key="tab.id"
          type="button"
          class="fd-repository-tab"
          :class="{ 'fd-repository-tab--active': activeTab === tab.id }"
          @click="activeTab = tab.id"
        >
          {{ tab.label }}
        </button>
      </nav>

      <div class="fd-repository-content">
        <template v-if="activeTab === 'overview'">
          <section class="fd-repository-section">
            <h2>{{ t('repositories.details') }}</h2>
            <dl class="fd-repository-facts">
              <div><dt>{{ t('repositories.columns.path') }}</dt><dd>{{ repo.checkouts.map((checkout) => checkout.path).join(', ') || '—' }}</dd></div>
              <div><dt>{{ t('repositories.columns.branch') }}</dt><dd>{{ repo.defaultBranch ?? '—' }}</dd></div>
              <div><dt>{{ t('repositories.columns.provider') }}</dt><dd>{{ repo.provider }}</dd></div>
              <div><dt>{{ t('repositories.columns.project') }}</dt><dd>{{ repo.projects.map((project) => project.name).join(', ') || '—' }}</dd></div>
              <div><dt>Remote</dt><dd>{{ repo.cloneUrl ?? repo.htmlUrl ?? '—' }}</dd></div>
              <div><dt>{{ t('repositories.changes', { count: diff.filesChanged }) }}</dt><dd>{{ status.dirty ? t('repositories.dirty') : t('repositories.clean') }}</dd></div>
            </dl>
          </section>

          <section class="fd-repository-section">
            <h2>{{ t('repositories.diff') }}</h2>
            <div class="fd-repository-diff">
              <span>{{ t('repositories.changes', { count: diff.filesChanged }) }}</span>
              <span class="fd-diff-add">+{{ diff.insertions }}</span>
              <span class="fd-diff-del">−{{ diff.deletions }}</span>
            </div>
            <pre v-if="diff.summary" class="fd-repository-diff-text">{{ diff.summary }}</pre>
            <p v-else class="fd-repository-muted">{{ t('repositories.noDiff') }}</p>
          </section>

          <section class="fd-repository-section">
            <h2>{{ t('repositories.branches') }}</h2>
            <div class="fd-branch-list">
              <div v-for="branch in branches" :key="branch.name" class="fd-branch-row">
                <span>
                  <GitBranch :size="12" aria-hidden="true" />
                  {{ branch.name }}
                  <em v-if="branch.current">{{ t('repositories.current') }}</em>
                </span>
                <FdButton
                  v-if="!branch.current"
                  variant="ghost"
                  size="sm"
                  :disabled="status.dirty"
                  @click="switchBranch(branch.name)"
                >
                  {{ t('repositories.switchBranch') }}
                </FdButton>
              </div>
            </div>
            <p v-if="status.dirty" class="fd-repository-warning">{{ t('repositories.dirtyGuard') }}</p>
          </section>
        </template>

        <template v-else-if="activeTab === 'branches'">
          <section class="fd-repository-section">
            <h2>{{ t('repositories.branches') }}</h2>
            <div class="fd-branch-list">
              <div v-for="branch in branches" :key="branch.name" class="fd-branch-row">
                <span>
                  <GitBranch :size="12" aria-hidden="true" />
                  {{ branch.name }}
                  <em v-if="branch.current">{{ t('repositories.current') }}</em>
                </span>
                <FdButton
                  v-if="!branch.current"
                  variant="ghost"
                  size="sm"
                  :disabled="status.dirty"
                  @click="switchBranch(branch.name)"
                >
                  {{ t('repositories.switchBranch') }}
                </FdButton>
              </div>
            </div>
            <p v-if="status.dirty" class="fd-repository-warning">{{ t('repositories.dirtyGuard') }}</p>
          </section>

          <section class="fd-repository-section">
            <h2>{{ t('repositories.createBranch') }}</h2>
            <form class="fd-branch-form" @submit.prevent="createBranch">
              <FdInput
                v-model="newBranch"
                :placeholder="t('repositories.branchNamePlaceholder')"
                :aria-label="t('repositories.branchName')"
                :disabled="status.dirty"
              />
              <FdButton
                variant="primary"
                size="md"
                type="submit"
                :disabled="status.dirty || !newBranch.trim()"
              >
                <Plus :size="13" aria-hidden="true" />
                {{ t('repositories.createBranchAction') }}
              </FdButton>
            </form>
            <p v-if="status.dirty" class="fd-repository-warning">{{ t('repositories.dirtyGuard') }}</p>
          </section>
        </template>

        <template v-else>
          <section class="fd-repository-section">
            <h2>{{ t('repositories.commits') }}</h2>
            <div class="fd-commit-list">
              <div v-for="entry in commits" :key="entry.hash" class="fd-commit-row">
                <code>{{ entry.shortHash }}</code>
                <span>{{ entry.subject }}</span>
                <span class="fd-commit-author">{{ entry.author }}</span>
                <span class="fd-commit-time">{{ formatRelative(entry.committedAt, app.resolvedLocale) }}</span>
              </div>
            </div>
          </section>
        </template>
      </div>
    </template>
  </div>
</template>

<style scoped>
.fd-repository-page {
  display: flex;
  flex-direction: column;
  width: 100%;
  height: 100%;
  min-width: 0;
  min-height: 0;
  overflow: hidden;
}

.fd-repository-back {
  padding: 8px var(--fd-content-pad) 0;
}

.fd-repository-state {
  display: flex;
  justify-content: center;
  padding: var(--fd-space-6) var(--fd-content-pad);
}

.fd-repository-status {
  display: inline-flex;
  align-items: center;
  height: 24px;
  padding: 0 8px;
  border: 1px solid var(--fd-border);
  border-radius: var(--fd-radius-pill);
  color: var(--fd-text-muted);
  font-size: var(--fd-text-11);
}

.fd-repository-status--dirty {
  border-color: var(--fd-warning);
  color: var(--fd-warning);
}

.fd-repository-meta {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 0 var(--fd-content-pad) 10px;
  color: var(--fd-text-muted);
  font-size: var(--fd-text-11);
}

.fd-repository-tabs {
  display: flex;
  flex: none;
  gap: 2px;
  padding: 0 var(--fd-content-pad);
  border-bottom: 1px solid var(--fd-border);
}

.fd-repository-tab {
  height: 34px;
  padding: 0 10px;
  border: 0;
  border-bottom: 2px solid transparent;
  background: transparent;
  color: var(--fd-text-muted);
  font-size: var(--fd-text-12);
  cursor: pointer;
}

.fd-repository-tab--active {
  border-bottom-color: var(--fd-accent);
  color: var(--fd-text-primary);
}

.fd-repository-content {
  flex: 1;
  min-height: 0;
  padding: var(--fd-space-4) var(--fd-content-pad) var(--fd-space-6);
  overflow: auto;
}

.fd-repository-section {
  padding-bottom: var(--fd-space-4);
  border-bottom: 1px solid var(--fd-border-subtle);
}

.fd-repository-section + .fd-repository-section {
  padding-top: var(--fd-space-4);
}

.fd-repository-section h2 {
  margin: 0 0 var(--fd-space-2);
  font-size: var(--fd-text-12);
  font-weight: var(--fd-weight-semibold);
}

.fd-repository-facts {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 8px 24px;
  margin: 0;
}

.fd-repository-facts div {
  display: flex;
  flex-direction: column;
  gap: 1px;
}

.fd-repository-facts dt {
  color: var(--fd-text-muted);
  font-size: var(--fd-text-11);
}

.fd-repository-facts dd {
  margin: 0;
  overflow: hidden;
  color: var(--fd-text-secondary);
  font-family: var(--fd-font-mono);
  font-size: var(--fd-text-11);
  text-overflow: ellipsis;
  white-space: nowrap;
}

.fd-repository-diff {
  display: flex;
  align-items: center;
  gap: 12px;
  color: var(--fd-text-secondary);
  font-size: var(--fd-text-12);
}

.fd-diff-add {
  color: var(--fd-success);
}

.fd-diff-del {
  color: var(--fd-danger);
}

.fd-repository-diff-text {
  margin: 8px 0 0;
  padding: 8px 10px;
  overflow: auto;
  border: 1px solid var(--fd-border-subtle);
  border-radius: var(--fd-radius-control);
  background: var(--fd-surface);
  color: var(--fd-text-secondary);
  font-family: var(--fd-font-mono);
  font-size: var(--fd-text-11);
}

.fd-branch-list {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.fd-branch-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  min-height: 32px;
  padding: 0 4px;
  border-bottom: 1px solid var(--fd-border-subtle);
  color: var(--fd-text-secondary);
  font-size: var(--fd-text-12);
}

.fd-branch-row span {
  display: flex;
  align-items: center;
  gap: 5px;
}

.fd-branch-row em {
  color: var(--fd-accent);
  font-size: var(--fd-text-10);
  font-style: normal;
}

.fd-repository-warning {
  margin: 8px 0 0;
  color: var(--fd-warning);
  font-size: var(--fd-text-11);
}

.fd-repository-muted {
  margin: 0;
  color: var(--fd-text-muted);
  font-size: var(--fd-text-11);
}

.fd-branch-form {
  display: flex;
  gap: var(--fd-space-2);
  max-width: 520px;
}

.fd-commit-list {
  display: flex;
  flex-direction: column;
}

.fd-commit-row {
  display: grid;
  grid-template-columns: 70px minmax(0, 1fr) 140px 100px;
  gap: var(--fd-space-3);
  align-items: center;
  min-height: 32px;
  border-bottom: 1px solid var(--fd-border-subtle);
  color: var(--fd-text-secondary);
  font-size: var(--fd-text-11);
}

.fd-commit-row code {
  color: var(--fd-accent);
}

.fd-commit-author,
.fd-commit-time {
  color: var(--fd-text-muted);
}

@media (max-width: 899px) {
  .fd-repository-facts {
    grid-template-columns: 1fr;
  }

  .fd-commit-row {
    grid-template-columns: 60px minmax(0, 1fr);
  }

  .fd-commit-author,
  .fd-commit-time {
    display: none;
  }
}
.fd-repository-attach {
  display: flex;
  gap: var(--fd-space-2);
  align-items: center;
  margin-top: var(--fd-space-3);
}
.fd-repository-error { color: var(--fd-danger); font-size: var(--fd-text-12); }
</style>
