<script setup lang="ts">
import { FolderGit2, Plus } from '@lucide/vue'
import { computed, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRouter } from 'vue-router'

import PageHeader from '@/components/layout/PageHeader.vue'
import PageToolbar from '@/components/layout/PageToolbar.vue'
import FdButton from '@/components/ui/FdButton.vue'
import FdEmptyState from '@/components/ui/FdEmptyState.vue'
import FdErrorState from '@/components/ui/FdErrorState.vue'
import FdInput from '@/components/ui/FdInput.vue'
import FdLoadingSkeleton from '@/components/ui/FdLoadingSkeleton.vue'
import { formatRelative } from '@/lib/format'
import { useAppStore } from '@/stores/app'
import { useGitStore } from '@/stores/git'
import { usePlanningStore } from '@/stores/planning'
import type { Repository } from '@/types/ipc'

import AddRepositoryDialog from './AddRepositoryDialog.vue'

const git = useGitStore()
const planning = usePlanningStore()
const app = useAppStore()
const router = useRouter()
const { t } = useI18n()

const search = ref('')
const dialogOpen = ref(false)
const filtered = computed(() => {
  const needle = search.value.trim().toLowerCase()
  if (!needle) {
    return git.repositories
  }
  return git.repositories.filter(
    (repository) =>
      repository.name.toLowerCase().includes(needle) ||
      repository.path.toLowerCase().includes(needle),
  )
})

function providerLabel(repository: Repository): string {
  return t(`repositories.provider${repository.provider[0]!.toUpperCase()}${repository.provider.slice(1)}`)
}

function open(repository: Repository): void {
  void router.push(`/repositories/${repository.id}`)
}

function onSaved(repository: Repository): void {
  void router.push(`/repositories/${repository.id}`)
}

onMounted(async () => {
  await Promise.all([
    git.loadRepositories(),
    git.loadAvailability(),
    planning.loadProjects(),
  ])
})
</script>

<template>
  <div class="fd-repositories-page">
    <PageHeader
      :title="t('pages.repositories.title')"
      :description="t('pages.repositories.description')"
    >
      <template #actions>
        <FdButton variant="primary" @click="dialogOpen = true">
          <Plus :size="14" aria-hidden="true" />
          {{ t('repositories.newRepository') }}
        </FdButton>
      </template>
    </PageHeader>

    <PageToolbar>
      <template #left>
        <FdInput
          v-model="search"
          type="search"
          class="fd-repositories-search"
          :placeholder="t('repositories.searchPlaceholder')"
          :aria-label="t('repositories.searchPlaceholder')"
        />
      </template>
      <template #right>
        <span v-if="git.availability" class="fd-git-availability">
          {{
            git.availability.available
              ? t('integrations.gitAvailable')
              : t('integrations.gitMissing')
          }}
        </span>
      </template>
    </PageToolbar>

    <div v-if="git.availability && !git.availability.available" class="fd-git-warning">
      {{ t('repositories.gitMissing') }}
    </div>

    <div class="fd-repositories-scroll">
      <div v-if="git.loading" class="fd-repositories-state">
        <FdLoadingSkeleton :rows="6" :height="44" />
      </div>
      <div v-else-if="git.error" class="fd-repositories-state">
        <FdErrorState
          :title="t('repositories.addError')"
          :message="git.error.message"
          :code="git.error.code"
          :detail="git.error.detail"
          :retry-label="t('common.retry')"
          @retry="git.loadRepositories()"
        />
      </div>
      <div v-else-if="filtered.length === 0" class="fd-repositories-state">
        <FdEmptyState
          :icon="FolderGit2"
          :title="t('repositories.emptyTitle')"
          :body="t('repositories.emptyBody')"
        >
          <FdButton variant="primary" @click="dialogOpen = true">
            <Plus :size="14" aria-hidden="true" />
            {{ t('repositories.addFirst') }}
          </FdButton>
        </FdEmptyState>
      </div>
      <div v-else class="fd-repository-table" role="table" :aria-label="t('pages.repositories.title')">
        <div class="fd-repository-grid fd-repository-grid--head" role="row">
          <span role="columnheader">{{ t('repositories.columns.repository') }}</span>
          <span role="columnheader">{{ t('repositories.columns.provider') }}</span>
          <span role="columnheader">{{ t('repositories.columns.branch') }}</span>
          <span role="columnheader">{{ t('repositories.columns.path') }}</span>
          <span role="columnheader">{{ t('repositories.columns.project') }}</span>
          <span role="columnheader">{{ t('repositories.columns.lastChecked') }}</span>
        </div>
        <div
          v-for="repository in filtered"
          :key="repository.id"
          class="fd-repository-grid fd-repository-row"
          role="row"
          tabindex="0"
          :aria-label="repository.name"
          @click="open(repository)"
          @keydown.enter="open(repository)"
        >
          <span class="fd-repository-row__name" role="cell">
            <strong>{{ repository.name }}</strong>
            <small>{{ repository.remoteUrl ?? repository.path }}</small>
          </span>
          <span role="cell">{{ providerLabel(repository) }}</span>
          <span role="cell">{{ repository.defaultBranch ?? '—' }}</span>
          <span class="fd-repository-row__path" role="cell">{{ repository.path }}</span>
          <span role="cell">{{ repository.projectName ?? '—' }}</span>
          <span class="fd-repository-row__muted" role="cell">
            {{
              repository.lastCheckedAt
                ? formatRelative(repository.lastCheckedAt, app.resolvedLocale)
                : '—'
            }}
          </span>
        </div>
      </div>
    </div>

    <AddRepositoryDialog
      v-model:open="dialogOpen"
      :projects="planning.projects"
      @saved="onSaved"
    />
  </div>
</template>

<style scoped>
.fd-repositories-page {
  display: flex;
  flex-direction: column;
  width: 100%;
  height: 100%;
  min-width: 0;
  min-height: 0;
  overflow: hidden;
}

.fd-repositories-search {
  width: 280px;
}

.fd-git-availability {
  color: var(--fd-text-muted);
  font-size: var(--fd-text-11);
}

.fd-git-warning {
  padding: 8px var(--fd-content-pad);
  border-bottom: 1px solid var(--fd-warning);
  background: color-mix(in srgb, var(--fd-warning) 10%, transparent);
  color: var(--fd-warning);
  font-size: var(--fd-text-11);
}

.fd-repositories-scroll {
  flex: 1;
  min-height: 0;
  overflow: auto;
}

.fd-repositories-state {
  display: flex;
  justify-content: center;
  padding: var(--fd-space-6) var(--fd-content-pad);
}

.fd-repository-grid {
  display: grid;
  grid-template-columns: minmax(220px, 1.4fr) 100px 130px minmax(240px, 1.6fr) 130px 110px;
  gap: var(--fd-space-3);
  align-items: center;
  min-height: 48px;
  padding: 0 var(--fd-content-pad);
}

.fd-repository-grid--head {
  position: sticky;
  top: 0;
  z-index: 2;
  min-height: 36px;
  border-bottom: 1px solid var(--fd-border);
  background: var(--fd-bg);
  color: var(--fd-text-muted);
  font-size: var(--fd-text-11);
}

.fd-repository-row {
  border-bottom: 1px solid var(--fd-border-subtle);
  cursor: pointer;
}

.fd-repository-row:hover {
  background: var(--fd-surface-hover);
}

.fd-repository-row__name {
  display: flex;
  flex-direction: column;
  gap: 1px;
  min-width: 0;
}

.fd-repository-row__name strong {
  font-size: var(--fd-text-12);
  font-weight: var(--fd-weight-medium);
}

.fd-repository-row__name small,
.fd-repository-row__muted {
  overflow: hidden;
  color: var(--fd-text-muted);
  font-size: var(--fd-text-11);
  text-overflow: ellipsis;
  white-space: nowrap;
}

.fd-repository-row__path {
  overflow: hidden;
  color: var(--fd-text-secondary);
  font-family: var(--fd-font-mono);
  font-size: var(--fd-text-11);
  text-overflow: ellipsis;
  white-space: nowrap;
}

@media (max-width: 1199px) {
  .fd-repository-grid {
    grid-template-columns: minmax(220px, 1.4fr) 90px 120px minmax(200px, 1.4fr) 100px;
  }

  .fd-repository-grid > :nth-child(6) {
    display: none;
  }
}
</style>
