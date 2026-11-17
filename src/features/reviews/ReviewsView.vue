<script setup lang="ts">
import { Eye } from '@lucide/vue'
import { computed, onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRouter } from 'vue-router'

import PageHeader from '@/components/layout/PageHeader.vue'
import FdButton from '@/components/ui/FdButton.vue'
import FdEmptyState from '@/components/ui/FdEmptyState.vue'
import FdErrorState from '@/components/ui/FdErrorState.vue'
import FdLoadingSkeleton from '@/components/ui/FdLoadingSkeleton.vue'
import GitHubRepoDialog from '@/features/integrations/GitHubRepoDialog.vue'
import { api, toIpcError } from '@/lib/ipc'
import { formatRelative } from '@/lib/format'
import { useAppStore } from '@/stores/app'
import { useGithubAuthStore } from '@/stores/githubAuth'
import type { GitHubSearchIssue } from '@/types/ipc'

const auth = useGithubAuthStore()
const app = useAppStore()
const router = useRouter()
const { t } = useI18n()

const reviews = ref<GitHubSearchIssue[]>([])
const loading = ref(false)
const error = ref<ReturnType<typeof toIpcError> | null>(null)
const repoDialog = ref(false)
const selectedRepo = computed(() => auth.selectedRepo)

async function load(): Promise<void> {
  if (!selectedRepo.value) return
  loading.value = true
  error.value = null
  try {
    reviews.value = await api.githubReviewRequested(
      selectedRepo.value.owner,
      selectedRepo.value.name,
    )
  } catch (caught) {
    error.value = toIpcError(caught)
  } finally {
    loading.value = false
  }
}

onMounted(async () => {
  await auth.loadConnection()
  await load()
})

watch(selectedRepo, async () => {
  await load()
})

function open(item: GitHubSearchIssue): void {
  void router.push({ path: '/pull-requests', query: { pr: item.number } })
}
</script>

<template>
  <div class="fd-reviews-page">
    <PageHeader :title="t('pages.reviews.title')" :description="t('pages.reviews.description')">
      <template #actions>
        <FdButton variant="ghost" @click="repoDialog = true">
          {{ selectedRepo?.fullName ?? t('reviewsPage.chooseRepo') }}
        </FdButton>
      </template>
    </PageHeader>

    <div v-if="auth.connectionLoading || !auth.connectionChecked" class="fd-reviews-state">
      <FdLoadingSkeleton :rows="5" :height="42" />
    </div>
    <div v-else-if="auth.storageUnavailable || auth.connectionCheckFailed" class="fd-reviews-state">
      <FdEmptyState
        :icon="Eye"
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
    <div v-else-if="!auth.connected" class="fd-reviews-state">
      <FdEmptyState :icon="Eye" :title="t('reviewsPage.connectTitle')" :body="t('reviewsPage.connectBody')">
        <FdButton variant="primary" @click="router.push('/integrations')">
          {{ t('reviewsPage.connectTitle') }}
        </FdButton>
      </FdEmptyState>
    </div>
    <div v-else-if="!selectedRepo" class="fd-reviews-state">
      <FdEmptyState :icon="Eye" :title="t('reviewsPage.noRepoTitle')" :body="t('reviewsPage.noRepoBody')">
        <FdButton variant="primary" @click="repoDialog = true">{{ t('reviewsPage.chooseRepo') }}</FdButton>
      </FdEmptyState>
    </div>
    <div v-else class="fd-reviews-scroll">
      <div v-if="loading" class="fd-reviews-state">
        <FdLoadingSkeleton :rows="6" :height="44" />
      </div>
      <div v-else-if="error" class="fd-reviews-state">
        <FdErrorState
          :title="t('reviewsPage.loadError')"
          :message="error.message"
          :code="error.code"
          :detail="error.detail"
          @retry="load"
        />
      </div>
      <FdEmptyState
        v-else-if="reviews.length === 0"
        :icon="Eye"
        :title="t('reviewsPage.empty')"
        :body="t('reviewsPage.emptyBody')"
      />
      <div v-else class="fd-reviews-table" role="table" :aria-label="t('pages.reviews.title')">
        <div class="fd-reviews-grid fd-reviews-grid--head" role="row">
          <span>{{ t('reviewsPage.columns.pr') }}</span>
          <span>{{ t('reviewsPage.columns.repository') }}</span>
          <span>{{ t('reviewsPage.columns.author') }}</span>
          <span>{{ t('reviewsPage.columns.state') }}</span>
          <span>{{ t('reviewsPage.columns.updated') }}</span>
        </div>
        <div
          v-for="item in reviews"
          :key="item.id"
          class="fd-reviews-grid fd-reviews-row"
          role="row"
          tabindex="0"
          @click="open(item)"
          @keydown.enter="open(item)"
        >
          <span class="fd-reviews-title"><strong>#{{ item.number }} · {{ item.title }}</strong></span>
          <span>{{ selectedRepo.fullName }}</span>
          <span>{{ item.user?.login ?? '—' }}</span>
          <span class="fd-reviews-cap">{{ item.state }}</span>
          <span class="fd-reviews-muted">{{ formatRelative(item.updatedAt, app.resolvedLocale) }}</span>
        </div>
      </div>
    </div>

    <GitHubRepoDialog v-model:open="repoDialog" />
  </div>
</template>

<style scoped>
.fd-reviews-page { display: flex; flex-direction: column; width: 100%; height: 100%; min-width: 0; min-height: 0; overflow: hidden; }
.fd-reviews-scroll { flex: 1; min-height: 0; overflow: auto; }
.fd-reviews-state { display: flex; justify-content: center; padding: var(--fd-space-6) var(--fd-content-pad); }
.fd-reviews-grid { display: grid; grid-template-columns: minmax(280px, 2fr) 180px 120px 80px 100px; gap: var(--fd-space-2); align-items: center; min-height: 46px; padding: 0 var(--fd-content-pad); }
.fd-reviews-grid--head { position: sticky; top: 0; z-index: 2; min-height: 32px; border-bottom: 1px solid var(--fd-border); background: var(--fd-bg); color: var(--fd-text-muted); font-size: var(--fd-text-11); }
.fd-reviews-row { border-bottom: 1px solid var(--fd-border-subtle); cursor: pointer; }
.fd-reviews-row:hover { background: var(--fd-surface-hover); }
.fd-reviews-title strong { overflow: hidden; font-size: var(--fd-text-12); font-weight: var(--fd-weight-medium); text-overflow: ellipsis; white-space: nowrap; }
.fd-reviews-muted { color: var(--fd-text-muted); font-size: var(--fd-text-11); }
.fd-reviews-cap { text-transform: capitalize; }
</style>
