<script setup lang="ts">
import { FolderGit2, Plus } from '@lucide/vue'
import { computed, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRouter } from 'vue-router'

import FdButton from '@/components/ui/FdButton.vue'
import FdConfirmDialog from '@/components/ui/FdConfirmDialog.vue'
import FdEmptyState from '@/components/ui/FdEmptyState.vue'
import FdErrorState from '@/components/ui/FdErrorState.vue'
import FdLoadingSkeleton from '@/components/ui/FdLoadingSkeleton.vue'
import FdSelect from '@/components/ui/FdSelect.vue'
import { useGitStore } from '@/stores/git'
import type { Repository } from '@/types/ipc'

const props = defineProps<{ projectId: string }>()
const git = useGitStore()
const router = useRouter()
const { t } = useI18n()

const linkId = ref('')
const unlinkTarget = ref<Repository | null>(null)
const unlinkOpen = ref(false)
const linking = ref(false)

const available = computed(() =>
  git.repositories.filter(
    (repository) => !git.projectRepositories.some((item) => item.id === repository.id),
  ),
)

async function load(): Promise<void> {
  await Promise.all([
    git.loadRepositories(),
    git.loadProjectRepositories(props.projectId),
  ])
}

async function link(): Promise<void> {
  if (!linkId.value || linking.value) return
  linking.value = true
  const ok = await git.linkProjectRepository(linkId.value, props.projectId)
  linking.value = false
  if (ok) linkId.value = ''
}

function askUnlink(repository: Repository): void {
  unlinkTarget.value = repository
  unlinkOpen.value = true
}

async function unlink(): Promise<void> {
  const target = unlinkTarget.value
  if (!target) return
  const ok = await git.unlinkProjectRepository(target.id, props.projectId)
  if (ok) {
    unlinkOpen.value = false
    unlinkTarget.value = null
  }
}

function providerLabel(repository: Repository): string {
  return repository.provider === 'local'
    ? t('repositories.providerLocal')
    : repository.provider.toUpperCase()
}

onMounted(load)
</script>

<template>
  <section class="fd-project-repositories">
    <div v-if="git.projectRepositoriesLoading" class="fd-project-state">
      <FdLoadingSkeleton :rows="3" :height="40" />
    </div>
    <div v-else-if="git.error" class="fd-project-state">
      <FdErrorState
        :title="t('projects.repositories.loadError')"
        :message="git.error.message"
        :code="git.error.code"
        :detail="git.error.detail"
        :retry-label="t('common.retry')"
        @retry="load"
      />
    </div>
    <template v-else>
      <div v-if="git.projectRepositories.length === 0" class="fd-project-state">
        <FdEmptyState
          :icon="FolderGit2"
          :title="t('projects.repositories.emptyTitle')"
          :body="t('projects.repositories.emptyBody')"
        />
      </div>
      <div v-else class="fd-project-repository-list">
        <article
          v-for="repository in git.projectRepositories"
          :key="repository.id"
          class="fd-project-repository"
        >
          <div class="fd-project-repository__main">
            <strong>{{ repository.fullName }}</strong>
            <span>
              {{ providerLabel(repository) }} · {{ repository.defaultBranch ?? '—' }}
              <template v-if="repository.checkouts.length">
                · {{ repository.checkouts.map((checkout) => checkout.path).join(', ') }}
              </template>
              <template v-else> · {{ t('repositories.remoteOnly') }}</template>
            </span>
            <small v-if="repository.description">{{ repository.description }}</small>
          </div>
          <div class="fd-project-repository__actions">
            <FdButton variant="ghost" size="sm" @click="router.push(`/repositories/${repository.id}`)">
              {{ t('repositories.open') }}
            </FdButton>
            <FdButton variant="ghost" size="sm" @click="askUnlink(repository)">
              {{ t('projects.repositories.unlink') }}
            </FdButton>
          </div>
        </article>
      </div>

      <div class="fd-project-repository-link">
        <FdSelect v-model="linkId" :aria-label="t('projects.repositories.linkExisting')">
          <option value="">{{ t('projects.repositories.choose') }}</option>
          <option v-for="repository in available" :key="repository.id" :value="repository.id">
            {{ repository.fullName }}
          </option>
        </FdSelect>
        <FdButton variant="secondary" :disabled="!linkId || linking" @click="link">
          <Plus :size="14" aria-hidden="true" />
          {{ t('projects.repositories.link') }}
        </FdButton>
        <FdButton variant="ghost" @click="router.push('/repositories')">
          {{ t('repositories.newRepository') }}
        </FdButton>
        <FdButton variant="ghost" @click="router.push('/integrations')">
          {{ t('repositories.discover') }}
        </FdButton>
      </div>
    </template>

    <FdConfirmDialog
      v-model="unlinkOpen"
      :title="t('projects.repositories.unlinkTitle')"
      :body="t('projects.repositories.unlinkBody', { name: unlinkTarget?.fullName ?? '' })"
      :confirm-label="t('projects.repositories.unlink')"
      @confirm="unlink"
    />
  </section>
</template>

<style scoped>
.fd-project-repositories { display: flex; flex-direction: column; gap: var(--fd-space-3); }
.fd-project-repository-list { display: flex; flex-direction: column; border-top: 1px solid var(--fd-border-subtle); }
.fd-project-repository { display: flex; align-items: flex-start; justify-content: space-between; gap: var(--fd-space-4); padding: 12px 0; border-bottom: 1px solid var(--fd-border-subtle); }
.fd-project-repository__main { display: flex; min-width: 0; flex-direction: column; gap: 3px; }
.fd-project-repository__main strong { font-size: var(--fd-text-13); }
.fd-project-repository__main span, .fd-project-repository__main small { color: var(--fd-text-muted); font-size: var(--fd-text-11); overflow-wrap: anywhere; }
.fd-project-repository__actions { display: flex; flex: none; gap: var(--fd-space-1); }
.fd-project-repository-link { display: flex; flex-wrap: wrap; gap: var(--fd-space-2); align-items: center; padding-top: var(--fd-space-2); }
</style>
