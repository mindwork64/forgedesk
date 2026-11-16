<script setup lang="ts">
import { FolderGit2 } from '@lucide/vue'
import { computed, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRouter } from 'vue-router'

import FdButton from '@/components/ui/FdButton.vue'
import FdEmptyState from '@/components/ui/FdEmptyState.vue'
import FdInput from '@/components/ui/FdInput.vue'
import FdLoadingSkeleton from '@/components/ui/FdLoadingSkeleton.vue'
import FdModal from '@/components/ui/FdModal.vue'
import FdSelect from '@/components/ui/FdSelect.vue'
import { useGithubAuthStore } from '@/stores/githubAuth'
import type { GitHubImportResult, Repository } from '@/types/ipc'

const open = defineModel<boolean>('open', { default: false })
const auth = useGithubAuthStore()
const router = useRouter()
const { t } = useI18n()
const search = ref('')
const results = ref<Record<string, GitHubImportResult>>({})
const targets = ref<Record<string, string>>({})
const nextPages = ref<Record<string, number>>({})
const importingId = ref<string | null>(null)

const filtered = computed(() => {
  const needle = search.value.trim().toLowerCase()
  if (!needle) {
    return auth.repositories
  }
  return auth.repositories.filter(
    (repo) =>
      repo.fullName.toLowerCase().includes(needle) ||
      (repo.description ?? '').toLowerCase().includes(needle),
  )
})

watch(open, (isOpen) => {
  if (isOpen) {
    search.value = ''
    results.value = {}
    void auth.discoverRepositories()
  }
})

function needsTarget(repo: Repository): boolean {
  return repo.projects.length > 1 && !targets.value[repo.id]
}

async function importRepo(repo: Repository, page = 1): Promise<void> {
  if (auth.importing) {
    return
  }
  const targetProjectId = targets.value[repo.id] ?? null
  importingId.value = repo.id
  const result = await auth.importIssues(repo.owner ?? '', repo.name, targetProjectId, page)
  importingId.value = null
  if (result) {
    results.value = { ...results.value, [repo.fullName]: result }
    if (result.nextPage) {
      nextPages.value = { ...nextPages.value, [repo.fullName]: result.nextPage }
    }
  }
}

async function useForPullRequests(repo: Repository): Promise<void> {
  await auth.selectRepository(repo)
  open.value = false
  void router.push('/pull-requests')
}

function openProject(repo: Repository): void {
  const result = results.value[repo.fullName]
  if (!result) {
    return
  }
  open.value = false
  void router.push(`/projects/${result.projectId}`)
}
</script>

<template>
  <FdModal v-model="open" :title="t('githubRepos.title')" size="lg">
    <div class="fd-github-repos">
      <p class="fd-github-repos__hint">{{ t('githubRepos.limitHint') }}</p>
      <p v-if="auth.discoveryHasMore" class="fd-github-repos__hint fd-github-repos__hint--warning">
        {{ t('githubRepos.discoveryPartial', { pages: auth.discoveryPages }) }}
        <FdButton
          v-if="auth.discoveryNextPage"
          variant="ghost"
          size="sm"
          :disabled="auth.discovering"
          @click="auth.discoverRepositories(auth.discoveryNextPage)"
        >
          {{ t('githubRepos.loadMore') }}
        </FdButton>
      </p>
      <FdInput
        v-model="search"
        type="search"
        :placeholder="t('githubRepos.searchPlaceholder')"
        :aria-label="t('githubRepos.searchPlaceholder')"
      />

      <div v-if="auth.discovering" class="fd-github-repos__state">
        <FdLoadingSkeleton :rows="5" :height="42" />
      </div>
      <div v-else-if="filtered.length === 0" class="fd-github-repos__state">
        <FdEmptyState
          :icon="FolderGit2"
          :title="t('githubRepos.noResults')"
          :body="t('githubRepos.limitHint')"
        />
      </div>
      <div v-else class="fd-github-repos__list">
        <div v-for="repo in filtered" :key="repo.id" class="fd-github-repo-row">
          <div class="fd-github-repo-row__main">
            <strong>{{ repo.fullName }}</strong>
            <span>
              {{ repo.visibility === 'private' ? t('githubRepos.private') : t('githubRepos.public') }}
              · {{ repo.defaultBranch ?? '—' }}
              <template v-if="repo.description"> · {{ repo.description }}</template>
            </span>
            <small v-if="results[repo.fullName]">
              {{
                t('githubRepos.imported', {
                  imported: results[repo.fullName]!.imported,
                  skipped: results[repo.fullName]!.skipped,
                })
              }}
              <span v-if="results[repo.fullName]!.hasMore">
                · {{ t('githubRepos.importPartial') }}
              </span>
            </small>
            <FdSelect
              v-if="repo.projects.length > 1"
              :model-value="targets[repo.id] ?? ''"
              :aria-label="t('githubRepos.targetProject')"
              @update:model-value="targets[repo.id] = $event"
              class="fd-github-repo-target"
            >
              <option value="">{{ t('githubRepos.chooseTarget') }}</option>
              <option v-for="project in repo.projects" :key="project.id" :value="project.id">
                {{ project.name }}
              </option>
            </FdSelect>
          </div>
          <div class="fd-github-repo-row__actions">
            <FdButton
              variant="ghost"
              size="sm"
              @click="useForPullRequests(repo)"
            >
              {{ t('githubRepos.useForPrs') }}
            </FdButton>
            <FdButton
              v-if="results[repo.fullName]"
              variant="ghost"
              size="sm"
              @click="openProject(repo)"
            >
              {{ t('githubRepos.open') }}
            </FdButton>
            <FdButton
              v-if="nextPages[repo.fullName]"
              variant="ghost"
              size="sm"
              :disabled="auth.importing"
              @click="importRepo(repo, nextPages[repo.fullName])"
            >
              {{ t('githubRepos.continue') }}
            </FdButton>
            <FdButton
              variant="primary"
              size="sm"
              :disabled="auth.importing || needsTarget(repo)"
              @click="importRepo(repo)"
            >
              {{ importingId === repo.id ? t('githubRepos.importing') : t('githubRepos.import') }}
            </FdButton>
          </div>
        </div>
      </div>
    </div>
  </FdModal>
</template>

<style scoped>
.fd-github-repos {
  display: flex;
  flex-direction: column;
  gap: var(--fd-space-3);
}

.fd-github-repos__hint {
  margin: 0;
  color: var(--fd-text-muted);
  font-size: var(--fd-text-11);
  line-height: var(--fd-line-body);
}

.fd-github-repos__state {
  padding: var(--fd-space-4);
}

.fd-github-repos__list {
  display: flex;
  flex-direction: column;
  max-height: 420px;
  overflow: auto;
}

.fd-github-repo-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--fd-space-3);
  min-height: 52px;
  padding: 6px 2px;
  border-bottom: 1px solid var(--fd-border-subtle);
}

.fd-github-repo-row__main {
  display: flex;
  flex-direction: column;
  gap: 1px;
  min-width: 0;
}

.fd-github-repo-row__main strong {
  font-size: var(--fd-text-12);
  font-weight: var(--fd-weight-medium);
}

.fd-github-repo-row__main span {
  overflow: hidden;
  color: var(--fd-text-muted);
  font-size: var(--fd-text-11);
  text-overflow: ellipsis;
  white-space: nowrap;
}

.fd-github-repo-row__main small {
  color: var(--fd-success);
  font-size: var(--fd-text-11);
}

.fd-github-repo-row__actions {
  display: flex;
  flex: none;
  align-items: center;
  gap: var(--fd-space-2);
}
</style>
