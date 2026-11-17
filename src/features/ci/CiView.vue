<script setup lang="ts">
import { Activity } from '@lucide/vue'
import { computed, onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRouter } from 'vue-router'

import PageHeader from '@/components/layout/PageHeader.vue'
import PageToolbar from '@/components/layout/PageToolbar.vue'
import FdButton from '@/components/ui/FdButton.vue'
import FdEmptyState from '@/components/ui/FdEmptyState.vue'
import FdErrorState from '@/components/ui/FdErrorState.vue'
import FdLoadingSkeleton from '@/components/ui/FdLoadingSkeleton.vue'
import GitHubRepoDialog from '@/features/integrations/GitHubRepoDialog.vue'
import { api, toIpcError } from '@/lib/ipc'
import { formatRelative } from '@/lib/format'
import { useAppStore } from '@/stores/app'
import { useGithubAuthStore } from '@/stores/githubAuth'
import type { CiOverview, GitHubCheckRun, GitHubCommit } from '@/types/ipc'

const auth = useGithubAuthStore()
const app = useAppStore()
const router = useRouter()
const { t } = useI18n()

const overview = ref<CiOverview | null>(null)
const checks = ref<GitHubCheckRun[]>([])
const selectedSha = ref<string | null>(null)
const loading = ref(false)
const checksLoading = ref(false)
const checksError = ref<string | null>(null)
const error = ref<ReturnType<typeof toIpcError> | null>(null)
const repoDialog = ref(false)
const selectedRepo = computed(() => auth.selectedRepo)

async function load(): Promise<void> {
  if (!selectedRepo.value) return
  loading.value = true
  error.value = null
  try {
    const result = await api.githubCiOverview(
      selectedRepo.value.owner,
      selectedRepo.value.name,
    )
    overview.value = result
    const first = result.commits[0]
    selectedSha.value = first?.sha ?? null
    checks.value = result.checks
  } catch (caught) {
    error.value = toIpcError(caught)
  } finally {
    loading.value = false
  }
}

async function selectCommit(commit: GitHubCommit): Promise<void> {
  if (!selectedRepo.value) return
  selectedSha.value = commit.sha
  checksLoading.value = true
  checksError.value = null
  try {
    checks.value = await api.githubCommitChecks(
      selectedRepo.value.owner,
      selectedRepo.value.name,
      commit.sha,
    )
  } catch (caught) {
    checksError.value = toIpcError(caught).message
  } finally {
    checksLoading.value = false
  }
}

onMounted(async () => {
  await auth.loadConnection()
  await load()
})

watch(selectedRepo, load)

function checkTone(check: GitHubCheckRun): string {
  if (check.conclusion === 'success') return 'ok'
  if (check.conclusion === 'failure' || check.conclusion === 'timed_out') return 'fail'
  return 'pending'
}

function duration(check: GitHubCheckRun): string {
  if (!check.startedAt || !check.completedAt) return '—'
  const ms = new Date(check.completedAt).getTime() - new Date(check.startedAt).getTime()
  return `${Math.max(0, Math.round(ms / 1000))}s`
}

function shortSha(sha: string): string {
  return sha.slice(0, 7)
}

function commitDate(commit: GitHubCommit): string {
  const date = commit.commit.author?.date
  return date ? formatRelative(date, app.resolvedLocale) : '—'
}
</script>

<template>
  <div class="fd-ci-page">
    <PageHeader :title="t('pages.ci.title')" :description="t('pages.ci.description')">
      <template #actions>
        <FdButton variant="ghost" @click="repoDialog = true">
          {{ selectedRepo?.fullName ?? t('ciPage.chooseRepo') }}
        </FdButton>
      </template>
    </PageHeader>

    <div v-if="auth.connectionLoading || !auth.connectionChecked" class="fd-ci-state">
      <FdLoadingSkeleton :rows="5" :height="42" />
    </div>
    <div v-else-if="auth.storageUnavailable || auth.connectionCheckFailed" class="fd-ci-state">
      <FdEmptyState
        :icon="Activity"
        :title="auth.storageUnavailable ? t('githubAuth.storageUnavailable') : t('githubAuth.connectionCheckFailed')"
        :body="auth.error?.message ?? t('githubAuth.connectionCheckFailedHint')"
      >
        <FdButton variant="secondary" @click="auth.loadConnection(true)">
          {{ t('githubAuth.retryCheck') }}
        </FdButton>
        <FdButton variant="ghost" @click="router.push('/integrations')">
          {{ t('nav.integrations') }}
        </FdButton>
      </FdEmptyState>
    </div>
    <div v-else-if="!auth.connected" class="fd-ci-state">
      <FdEmptyState :icon="Activity" :title="t('ciPage.connectTitle')" :body="t('ciPage.connectBody')">
        <FdButton variant="primary" @click="router.push('/integrations')">
          {{ t('ciPage.connectTitle') }}
        </FdButton>
      </FdEmptyState>
    </div>
    <div v-else-if="!selectedRepo" class="fd-ci-state">
      <FdEmptyState :icon="Activity" :title="t('ciPage.noRepoTitle')" :body="t('ciPage.noRepoBody')">
        <FdButton variant="primary" @click="repoDialog = true">{{ t('ciPage.chooseRepo') }}</FdButton>
      </FdEmptyState>
    </div>
    <template v-else>
      <PageToolbar>
        <template #left>
          <span class="fd-ci-branch">{{ overview?.defaultBranch ?? '—' }}</span>
        </template>
        <template #right>
          <FdButton variant="ghost" size="sm" @click="load">{{ t('ciPage.refresh') }}</FdButton>
        </template>
      </PageToolbar>
      <div class="fd-ci-content">
        <div v-if="loading" class="fd-ci-state">
          <FdLoadingSkeleton :rows="7" :height="44" />
        </div>
        <div v-else-if="error" class="fd-ci-state">
          <FdErrorState
            :title="t('ciPage.loadError')"
            :message="error.message"
            :code="error.code"
            :detail="error.detail"
            @retry="load"
          />
        </div>
        <template v-else-if="overview">
          <aside class="fd-ci-commits">
            <h2>{{ t('ciPage.commits') }}</h2>
            <div
              v-for="commit in overview.commits"
              :key="commit.sha"
              class="fd-ci-commit"
              :class="{ 'fd-ci-commit--active': commit.sha === selectedSha }"
              role="button"
              tabindex="0"
              @click="selectCommit(commit)"
              @keydown.enter="selectCommit(commit)"
            >
              <code>{{ shortSha(commit.sha) }}</code>
              <strong>{{ commit.commit.message.split('\n')[0] }}</strong>
              <small>
                {{ commit.commit.author?.name ?? commit.author?.login ?? '—' }} ·
                {{ commitDate(commit) }}
              </small>
            </div>
          </aside>
          <section class="fd-ci-checks">
            <div class="fd-ci-checks__head">
              <h2>{{ t('ciPage.checks') }} <code v-if="selectedSha">{{ shortSha(selectedSha) }}</code></h2>
            </div>
            <p v-if="checksError" class="fd-ci-error" role="alert">{{ checksError }}</p>
            <div v-if="checksLoading" class="fd-ci-state">
              <FdLoadingSkeleton :rows="4" :height="36" />
            </div>
            <FdEmptyState
              v-else-if="checks.length === 0"
              :icon="Activity"
              :title="t('ciPage.noChecks')"
              :body="t('ciPage.noChecksBody')"
            />
            <div v-else class="fd-ci-check-list">
              <div v-for="check in checks" :key="check.id" class="fd-ci-check-row">
                <span class="fd-ci-check" :class="`fd-ci-check--${checkTone(check)}`">
                  {{ check.conclusion ?? check.status }}
                </span>
                <span>{{ check.name }}</span>
                <span class="fd-ci-muted">{{ duration(check) }}</span>
              </div>
            </div>
          </section>
        </template>
      </div>
    </template>

    <GitHubRepoDialog v-model:open="repoDialog" />
  </div>
</template>

<style scoped>
.fd-ci-page { display: flex; flex-direction: column; width: 100%; height: 100%; min-width: 0; min-height: 0; overflow: hidden; }
.fd-ci-content { display: flex; flex: 1; min-width: 0; min-height: 0; }
.fd-ci-state { display: flex; justify-content: center; padding: var(--fd-space-6) var(--fd-content-pad); }
.fd-ci-branch { color: var(--fd-text-secondary); font-family: var(--fd-font-mono); font-size: var(--fd-text-11); }
.fd-ci-commits { width: 340px; min-width: 0; overflow: auto; border-right: 1px solid var(--fd-border); padding: var(--fd-space-2); }
.fd-ci-commits h2, .fd-ci-checks h2 { margin: 0; padding: 6px 8px; color: var(--fd-text-muted); font-size: var(--fd-text-11); font-weight: var(--fd-weight-medium); text-transform: uppercase; }
.fd-ci-commit { display: flex; flex-direction: column; gap: 2px; padding: 8px; border-radius: var(--fd-radius-control); cursor: pointer; }
.fd-ci-commit:hover, .fd-ci-commit--active { background: var(--fd-surface-hover); }
.fd-ci-commit code { color: var(--fd-text-muted); font-size: var(--fd-text-10); }
.fd-ci-commit strong { overflow: hidden; font-size: var(--fd-text-12); font-weight: var(--fd-weight-medium); text-overflow: ellipsis; white-space: nowrap; }
.fd-ci-commit small { overflow: hidden; color: var(--fd-text-muted); font-size: var(--fd-text-10); text-overflow: ellipsis; white-space: nowrap; }
.fd-ci-checks { flex: 1; min-width: 0; overflow: auto; padding: var(--fd-space-2) var(--fd-content-pad); }
.fd-ci-checks__head { border-bottom: 1px solid var(--fd-border-subtle); }
.fd-ci-checks h2 { display: flex; gap: 8px; align-items: center; padding-left: 0; }
.fd-ci-checks h2 code { font-size: var(--fd-text-10); text-transform: none; }
.fd-ci-check-list { display: flex; flex-direction: column; }
.fd-ci-check-row { display: grid; grid-template-columns: 90px minmax(0, 1fr) 50px; gap: var(--fd-space-2); align-items: center; min-height: 40px; border-bottom: 1px solid var(--fd-border-subtle); font-size: var(--fd-text-12); }
.fd-ci-check { font-size: var(--fd-text-10); text-transform: uppercase; }
.fd-ci-check--ok { color: var(--fd-success); }
.fd-ci-check--fail { color: var(--fd-danger); }
.fd-ci-check--pending { color: var(--fd-warning); }
.fd-ci-muted { color: var(--fd-text-muted); font-size: var(--fd-text-11); }
.fd-ci-error { margin: 8px 0; color: var(--fd-danger); font-size: var(--fd-text-11); }
</style>
