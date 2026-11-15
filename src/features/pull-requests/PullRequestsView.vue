<script setup lang="ts">
import { GitPullRequest, X } from '@lucide/vue'
import { computed, onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRouter } from 'vue-router'

import EntityDetailPane from '@/components/issues/EntityDetailPane.vue'
import PageHeader from '@/components/layout/PageHeader.vue'
import PageToolbar from '@/components/layout/PageToolbar.vue'
import FdButton from '@/components/ui/FdButton.vue'
import FdEmptyState from '@/components/ui/FdEmptyState.vue'
import FdErrorState from '@/components/ui/FdErrorState.vue'
import FdIconButton from '@/components/ui/FdIconButton.vue'
import FdLoadingSkeleton from '@/components/ui/FdLoadingSkeleton.vue'
import FdSegmented from '@/components/ui/FdSegmented.vue'
import GitHubRepoDialog from '@/features/integrations/GitHubRepoDialog.vue'
import { formatRelative } from '@/lib/format'
import { useAppStore } from '@/stores/app'
import { useGithubAuthStore } from '@/stores/githubAuth'
import { usePullRequestStore } from '@/stores/pullRequests'
import type { GitHubCheckRun, GitHubPull } from '@/types/ipc'

type DetailTab = 'overview' | 'commits' | 'checks' | 'reviews'

const auth = useGithubAuthStore()
const prs = usePullRequestStore()
const app = useAppStore()
const router = useRouter()
const { t } = useI18n()

const state = ref<'open' | 'closed' | 'all'>('open')
const repoDialog = ref(false)
const detailTab = ref<DetailTab>('overview')
const selectedRepo = computed(() => auth.selectedRepo)
const pull = computed(() => prs.detail?.pull ?? null)

const stateOptions = computed(() => [
  { value: 'open', label: t('prs.open') },
  { value: 'closed', label: t('prs.closed') },
  { value: 'all', label: t('prs.all') },
])
const tabs = computed(() => [
  { id: 'overview' as const, label: t('prs.detail.overview') },
  { id: 'commits' as const, label: t('prs.detail.commits') },
  { id: 'checks' as const, label: t('prs.detail.checks') },
  { id: 'reviews' as const, label: t('prs.detail.reviews') },
])

async function load(): Promise<void> {
  if (selectedRepo.value) {
    await prs.load(selectedRepo.value.owner, selectedRepo.value.name, state.value)
  }
}

onMounted(async () => {
  await auth.loadConnection()
  await load()
})

watch(state, load)
watch(selectedRepo, async () => {
  prs.close()
  await load()
})

function openPull(item: GitHubPull): void {
  if (!selectedRepo.value) return
  detailTab.value = 'overview'
  void prs.open(selectedRepo.value.owner, selectedRepo.value.name, item.number)
}

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
</script>

<template>
  <div class="fd-pr-page">
    <PageHeader
      :title="t('pages.pullRequests.title')"
      :description="t('pages.pullRequests.description')"
    >
      <template #actions>
        <FdButton variant="ghost" @click="repoDialog = true">
          {{ selectedRepo?.fullName ?? t('prs.chooseRepo') }}
        </FdButton>
      </template>
    </PageHeader>

    <PageToolbar v-if="selectedRepo">
      <template #left>
        <FdSegmented v-model="state" :options="stateOptions" :label="t('prs.columns.state')" />
      </template>
      <template #right>
        <FdButton variant="ghost" size="sm" @click="load">{{ t('prs.refresh') }}</FdButton>
      </template>
    </PageToolbar>

    <div class="fd-pr-content">
      <div v-if="!auth.connected" class="fd-pr-state">
        <FdEmptyState :icon="GitPullRequest" :title="t('prs.connectTitle')" :body="t('prs.connectBody')">
          <FdButton variant="primary" @click="router.push('/integrations')">
            {{ t('prs.connectTitle') }}
          </FdButton>
        </FdEmptyState>
      </div>
      <div v-else-if="!selectedRepo" class="fd-pr-state">
        <FdEmptyState :icon="GitPullRequest" :title="t('prs.noRepoTitle')" :body="t('prs.noRepoBody')">
          <FdButton variant="primary" @click="repoDialog = true">{{ t('prs.chooseRepo') }}</FdButton>
        </FdEmptyState>
      </div>
      <div v-else class="fd-pr-main">
        <div class="fd-pr-scroll">
          <div v-if="prs.loading" class="fd-pr-state">
            <FdLoadingSkeleton :rows="7" :height="44" />
          </div>
          <div v-else-if="prs.error" class="fd-pr-state">
            <FdErrorState
              :title="t('prs.loadError')"
              :message="prs.error.message"
              :code="prs.error.code"
              :detail="prs.error.detail"
              @retry="load"
            />
          </div>
          <FdEmptyState
            v-else-if="prs.pulls.length === 0"
            :icon="GitPullRequest"
            :title="t('prs.noPulls')"
            :body="t('prs.noRepoBody')"
          />
          <div v-else class="fd-pr-table" role="table" :aria-label="t('pages.pullRequests.title')">
            <div class="fd-pr-grid fd-pr-grid--head" role="row">
              <span>{{ t('prs.columns.pr') }}</span>
              <span>{{ t('prs.columns.repository') }}</span>
              <span>{{ t('prs.columns.state') }}</span>
              <span>{{ t('prs.columns.draft') }}</span>
              <span>{{ t('prs.columns.review') }}</span>
              <span>{{ t('prs.columns.checks') }}</span>
              <span>{{ t('prs.columns.branches') }}</span>
              <span>{{ t('prs.columns.updated') }}</span>
            </div>
            <div
              v-for="item in prs.pulls"
              :key="item.id"
              class="fd-pr-grid fd-pr-row"
              :class="{ 'fd-pr-row--active': prs.selectedNumber === item.number }"
              role="row"
              tabindex="0"
              @click="openPull(item)"
              @keydown.enter="openPull(item)"
            >
              <span class="fd-pr-title">
                <strong>#{{ item.number }} · {{ item.title }}</strong>
                <small>{{ item.user?.login ?? '—' }}</small>
              </span>
              <span>{{ selectedRepo.fullName }}</span>
              <span class="fd-pr-cap">{{ item.state }}</span>
              <span>{{ item.draft ? t('prs.draftBadge') : '—' }}</span>
              <span>—</span>
              <span>—</span>
              <span class="fd-pr-branch">{{ item.head.ref }} → {{ item.base.ref }}</span>
              <span class="fd-pr-muted">{{ formatRelative(item.updatedAt, app.resolvedLocale) }}</span>
            </div>
          </div>
        </div>

        <EntityDetailPane
          v-if="prs.selectedNumber !== null"
          :label="`#${prs.selectedNumber}`"
          @close="prs.close()"
        >
          <header class="fd-pr-detail-header">
            <div>
              <span class="fd-pr-muted">#{{ prs.selectedNumber }} · {{ selectedRepo.fullName }}</span>
              <strong>{{ pull?.title ?? '…' }}</strong>
            </div>
            <FdIconButton :label="t('common.close')" @click="prs.close()">
              <X :size="16" />
            </FdIconButton>
          </header>

          <div v-if="prs.detailLoading" class="fd-pr-detail-state">
            <FdLoadingSkeleton :rows="6" :height="40" />
          </div>
          <template v-else-if="prs.detail">
            <div class="fd-pr-meta">
              <span class="fd-pr-cap">{{ prs.detail.pull.state }}</span>
              <span v-if="prs.detail.pull.draft">{{ t('prs.draftBadge') }}</span>
              <span>{{ prs.detail.pull.user?.login ?? '—' }}</span>
            </div>
            <nav class="fd-pr-tabs">
              <button
                v-for="tab in tabs"
                :key="tab.id"
                type="button"
                class="fd-pr-tab"
                :class="{ 'fd-pr-tab--active': detailTab === tab.id }"
                @click="detailTab = tab.id"
              >
                {{ tab.label }}
              </button>
            </nav>
            <div class="fd-pr-detail-body">
              <template v-if="detailTab === 'overview'">
                <p v-if="prs.detail.pull.body" class="fd-pr-description">{{ prs.detail.pull.body }}</p>
                <p v-else class="fd-pr-muted">{{ t('prs.detail.noBody') }}</p>
                <dl class="fd-pr-facts">
                  <div><dt>{{ t('prs.detail.branches') }}</dt><dd>{{ prs.detail.pull.head.ref }} → {{ prs.detail.pull.base.ref }}</dd></div>
                  <div><dt>{{ t('prs.detail.changes') }}</dt><dd>+{{ prs.detail.pull.additions ?? 0 }} −{{ prs.detail.pull.deletions ?? 0 }} · {{ prs.detail.pull.changedFiles ?? 0 }} files</dd></div>
                  <div><dt>{{ t('prs.detail.mergeable') }}</dt><dd>{{ prs.detail.pull.mergeable === null ? t('prs.detail.unknown') : prs.detail.pull.mergeable }}</dd></div>
                  <div><dt>{{ t('prs.detail.reviewers') }}</dt><dd>{{ prs.detail.pull.requestedReviewers.map((r) => r.login).join(', ') || '—' }}</dd></div>
                </dl>
              </template>
              <template v-else-if="detailTab === 'commits'">
                <p v-if="prs.detail.commits.length === 0" class="fd-pr-muted">{{ t('prs.detail.noCommits') }}</p>
                <div v-for="commit in prs.detail.commits" :key="commit.sha" class="fd-pr-row-line">
                  <code>{{ commit.sha.slice(0, 7) }}</code>
                  <span>{{ commit.commit.message.split('\n')[0] }}</span>
                  <span>{{ commit.commit.author?.name ?? commit.author?.login ?? '—' }}</span>
                </div>
              </template>
              <template v-else-if="detailTab === 'checks'">
                <p v-if="prs.detail.checks.length === 0" class="fd-pr-muted">{{ t('prs.detail.noChecks') }}</p>
                <div v-for="check in prs.detail.checks" :key="check.id" class="fd-pr-row-line">
                  <span class="fd-pr-check" :class="`fd-pr-check--${checkTone(check)}`">{{ check.conclusion ?? check.status }}</span>
                  <span>{{ check.name }}</span>
                  <span class="fd-pr-muted">{{ duration(check) }}</span>
                </div>
              </template>
              <template v-else>
                <p v-if="prs.detail.reviews.length === 0" class="fd-pr-muted">{{ t('prs.detail.noReviews') }}</p>
                <div v-for="review in prs.detail.reviews" :key="review.id" class="fd-pr-review">
                  <strong>{{ review.user?.login ?? '—' }}</strong>
                  <span class="fd-pr-muted">{{ review.state }}</span>
                  <p>{{ review.body || '—' }}</p>
                </div>
              </template>
            </div>
          </template>
        </EntityDetailPane>
      </div>
    </div>

    <GitHubRepoDialog v-model:open="repoDialog" />
  </div>
</template>

<style scoped>
.fd-pr-page { display: flex; flex-direction: column; width: 100%; height: 100%; min-width: 0; min-height: 0; overflow: hidden; }
.fd-pr-content, .fd-pr-main { display: flex; flex: 1; min-width: 0; min-height: 0; }
.fd-pr-scroll { flex: 1; min-width: 0; min-height: 0; overflow: auto; }
.fd-pr-state { display: flex; justify-content: center; padding: var(--fd-space-6) var(--fd-content-pad); }
.fd-pr-grid { display: grid; grid-template-columns: minmax(280px, 2fr) 150px 70px 70px 70px 70px 160px 95px; gap: var(--fd-space-2); align-items: center; min-height: 46px; padding: 0 var(--fd-content-pad); }
.fd-pr-grid--head { position: sticky; top: 0; z-index: 2; min-height: 32px; border-bottom: 1px solid var(--fd-border); background: var(--fd-bg); color: var(--fd-text-muted); font-size: var(--fd-text-11); }
.fd-pr-row { border-bottom: 1px solid var(--fd-border-subtle); cursor: pointer; }
.fd-pr-row:hover, .fd-pr-row--active { background: var(--fd-surface-hover); }
.fd-pr-title { display: flex; flex-direction: column; gap: 1px; min-width: 0; }
.fd-pr-title strong { overflow: hidden; font-size: var(--fd-text-12); font-weight: var(--fd-weight-medium); text-overflow: ellipsis; white-space: nowrap; }
.fd-pr-title small, .fd-pr-muted { color: var(--fd-text-muted); font-size: var(--fd-text-11); }
.fd-pr-cap { text-transform: capitalize; }
.fd-pr-branch { overflow: hidden; color: var(--fd-text-muted); font-family: var(--fd-font-mono); font-size: var(--fd-text-11); text-overflow: ellipsis; white-space: nowrap; }
.fd-pr-detail-header { display: flex; align-items: flex-start; justify-content: space-between; gap: var(--fd-space-2); padding: var(--fd-space-3) var(--fd-space-4); border-bottom: 1px solid var(--fd-border-subtle); }
.fd-pr-detail-header > div { display: flex; flex-direction: column; gap: 2px; min-width: 0; }
.fd-pr-detail-header strong { overflow: hidden; font-size: var(--fd-text-15); font-weight: var(--fd-weight-semibold); text-overflow: ellipsis; }
.fd-pr-detail-state { padding: var(--fd-space-4); }
.fd-pr-meta { display: flex; gap: 8px; padding: var(--fd-space-2) var(--fd-space-4); color: var(--fd-text-muted); font-size: var(--fd-text-11); }
.fd-pr-tabs { display: flex; gap: 2px; padding: 0 var(--fd-space-4); border-bottom: 1px solid var(--fd-border-subtle); }
.fd-pr-tab { height: 32px; padding: 0 8px; border: 0; border-bottom: 2px solid transparent; background: transparent; color: var(--fd-text-muted); font-size: var(--fd-text-11); cursor: pointer; }
.fd-pr-tab--active { border-bottom-color: var(--fd-accent); color: var(--fd-text-primary); }
.fd-pr-detail-body { padding: var(--fd-space-3) var(--fd-space-4) var(--fd-space-5); }
.fd-pr-description { margin: 0 0 var(--fd-space-3); color: var(--fd-text-secondary); font-size: var(--fd-text-12); line-height: var(--fd-line-body); white-space: pre-wrap; }
.fd-pr-facts { display: flex; flex-direction: column; gap: 6px; margin: 0; }
.fd-pr-facts div { display: flex; flex-direction: column; gap: 1px; }
.fd-pr-facts dt { color: var(--fd-text-muted); font-size: var(--fd-text-11); }
.fd-pr-facts dd { margin: 0; color: var(--fd-text-secondary); font-size: var(--fd-text-11); }
.fd-pr-row-line { display: grid; grid-template-columns: 70px minmax(0, 1fr) 90px; gap: var(--fd-space-2); align-items: center; min-height: 30px; border-bottom: 1px solid var(--fd-border-subtle); color: var(--fd-text-secondary); font-size: var(--fd-text-11); }
.fd-pr-review { display: grid; grid-template-columns: 90px 90px minmax(0, 1fr); gap: var(--fd-space-2); min-height: 30px; border-bottom: 1px solid var(--fd-border-subtle); font-size: var(--fd-text-11); }
.fd-pr-review p { margin: 0; color: var(--fd-text-muted); }
.fd-pr-check--ok { color: var(--fd-success); }
.fd-pr-check--fail { color: var(--fd-danger); }
.fd-pr-check--pending { color: var(--fd-warning); }
@media (max-width: 1199px) { .fd-pr-grid { grid-template-columns: minmax(250px, 2fr) 120px 65px 130px 90px; } .fd-pr-grid > :nth-child(4), .fd-pr-grid > :nth-child(5), .fd-pr-grid > :nth-child(6) { display: none; } }
</style>
