<script setup lang="ts">
import { GitBranch, GitFork, Lock, Plug } from '@lucide/vue'
import { onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRouter } from 'vue-router'

import PageHeader from '@/components/layout/PageHeader.vue'
import FdButton from '@/components/ui/FdButton.vue'
import FdConfirmDialog from '@/components/ui/FdConfirmDialog.vue'
import { useGithubAuthStore } from '@/stores/githubAuth'
import { useGitStore } from '@/stores/git'
import GitHubDeviceDialog from './GitHubDeviceDialog.vue'
import GitHubRepoDialog from './GitHubRepoDialog.vue'

const git = useGitStore()
const auth = useGithubAuthStore()
const router = useRouter()
const { t } = useI18n()
const connectOpen = ref(false)
const disconnectOpen = ref(false)
const reposOpen = ref(false)

onMounted(async () => {
  await Promise.all([
    git.loadAvailability(),
    git.loadRepositories(),
    auth.loadConnection(),
  ])
})
</script>

<template>
  <div class="fd-integrations-page">
    <PageHeader
      :title="t('pages.integrations.title')"
      :description="t('pages.integrations.description')"
    />

    <div class="fd-integrations-content">
      <section class="fd-provider-card fd-provider-card--primary">
        <div class="fd-provider-card__head">
          <div class="fd-provider-card__identity">
            <span class="fd-provider-icon"><GitFork :size="20" aria-hidden="true" /></span>
            <div>
              <h2>{{ t('integrations.github') }}</h2>
              <p>{{ t('integrations.githubHint') }}</p>
            </div>
          </div>
          <span
            class="fd-provider-state"
            :class="{ 'fd-provider-state--ok': auth.connected }"
          >
            {{
              auth.connected
                ? t('apiMonitor.connectedAs', { login: auth.connection?.login })
                : t('integrations.notConnected')
            }}
          </span>
        </div>
        <p v-if="!auth.connected" class="fd-provider-card__notice">
          {{ t('githubAuth.hint') }}
        </p>
        <p v-else class="fd-provider-card__notice">
          {{ t('integrations.lastSync') }}: {{ t('integrations.never') }}
          <template v-if="auth.connection?.storageWarning">
            · {{ auth.connection.storageWarning }}
          </template>
        </p>
        <div class="fd-provider-permissions">
          <div>
            <span>{{ t('integrations.permissions') }}</span>
            <p>{{ t('integrations.readScope') }}</p>
            <p>{{ t('integrations.writeScope') }}</p>
          </div>
        </div>
        <div class="fd-provider-card__footer">
          <template v-if="auth.connected">
            <FdButton variant="ghost" size="sm" @click="auth.refresh()">
              {{ t('githubAuth.refresh') }}
            </FdButton>
            <FdButton variant="ghost" size="sm" @click="reposOpen = true">
              {{ t('githubRepos.discover') }}
            </FdButton>
            <FdButton variant="ghost" size="sm" @click="connectOpen = true">
              {{ t('githubAuth.reauthorize') }}
            </FdButton>
            <FdButton
              variant="ghost"
              size="sm"
              @click="disconnectOpen = true"
            >
              {{ t('integrations.disconnect') }}
            </FdButton>
          </template>
          <template v-else>
            <FdButton variant="primary" size="sm" @click="connectOpen = true">
              {{ t('integrations.connect') }}
            </FdButton>
            <span class="fd-provider-muted">{{ t('integrations.noAccount') }}</span>
          </template>
        </div>
      </section>

      <section class="fd-provider-card">
        <div class="fd-provider-card__head">
          <div class="fd-provider-card__identity">
            <span class="fd-provider-icon"><GitBranch :size="20" aria-hidden="true" /></span>
            <div>
              <h2>{{ t('integrations.localGit') }}</h2>
              <p>{{ t('integrations.localHint') }}</p>
            </div>
          </div>
          <span
            class="fd-provider-state"
            :class="{ 'fd-provider-state--ok': git.availability?.available }"
          >
            {{
              git.availability?.available
                ? t('integrations.gitAvailable')
                : t('integrations.gitMissing')
            }}
          </span>
        </div>
        <div class="fd-provider-card__footer">
          <span class="fd-provider-muted">
            {{
              git.availability?.version ??
              git.availability?.error ??
              t('common.loading')
            }}
          </span>
          <span class="fd-provider-muted">
            {{ t('integrations.configuredRepositories', { count: git.repositories.length }) }}
          </span>
          <FdButton variant="ghost" size="sm" @click="router.push('/repositories')">
            {{ t('integrations.manageRepositories') }}
          </FdButton>
        </div>
      </section>

      <section class="fd-provider-card fd-provider-card--planned">
        <div class="fd-provider-card__head">
          <div class="fd-provider-card__identity">
            <span class="fd-provider-icon"><Plug :size="20" aria-hidden="true" /></span>
            <div>
              <h2>{{ t('integrations.gitlab') }}</h2>
              <p>{{ t('integrations.planned') }}</p>
            </div>
          </div>
          <span class="fd-provider-state">{{ t('integrations.notConnected') }}</span>
        </div>
      </section>

      <section class="fd-provider-card fd-provider-card--planned">
        <div class="fd-provider-card__head">
          <div class="fd-provider-card__identity">
            <span class="fd-provider-icon"><Lock :size="20" aria-hidden="true" /></span>
            <div>
              <h2>{{ t('integrations.gitea') }}</h2>
              <p>{{ t('integrations.planned') }}</p>
            </div>
          </div>
          <span class="fd-provider-state">{{ t('integrations.notConnected') }}</span>
        </div>
      </section>
    </div>

    <GitHubDeviceDialog v-model:open="connectOpen" />
    <GitHubRepoDialog v-model:open="reposOpen" />

    <FdConfirmDialog
      v-model="disconnectOpen"
      :title="t('githubAuth.disconnectTitle')"
      :body="t('githubAuth.disconnectBody')"
      :confirm-label="t('githubAuth.disconnectConfirm')"
      destructive
      @confirm="auth.disconnect()"
    />
  </div>
</template>

<style scoped>
.fd-integrations-page {
  display: flex;
  flex-direction: column;
  width: 100%;
  height: 100%;
  min-width: 0;
  min-height: 0;
  overflow: auto;
}

.fd-integrations-content {
  display: grid;
  grid-template-columns: minmax(0, 1.4fr) minmax(0, 1fr);
  gap: var(--fd-space-4);
  padding: var(--fd-space-2) var(--fd-content-pad) var(--fd-space-6);
}

.fd-provider-card {
  display: flex;
  flex-direction: column;
  gap: var(--fd-space-3);
  padding: var(--fd-space-4);
  border: 1px solid var(--fd-border);
  border-radius: var(--fd-radius-card);
  background: var(--fd-surface);
}

.fd-provider-card--primary {
  grid-row: span 2;
}

.fd-provider-card--planned {
  opacity: 0.7;
}

.fd-provider-card__head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: var(--fd-space-3);
}

.fd-provider-card__identity {
  display: flex;
  align-items: flex-start;
  gap: var(--fd-space-2);
}

.fd-provider-icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 36px;
  border: 1px solid var(--fd-border);
  border-radius: var(--fd-radius-control);
  color: var(--fd-text-secondary);
}

.fd-provider-card h2 {
  margin: 0;
  font-size: var(--fd-text-14);
  font-weight: var(--fd-weight-semibold);
}

.fd-provider-card p {
  margin: 2px 0 0;
  color: var(--fd-text-muted);
  font-size: var(--fd-text-11);
  line-height: var(--fd-line-body);
}

.fd-provider-state {
  flex: none;
  color: var(--fd-text-muted);
  font-size: var(--fd-text-11);
}

.fd-provider-state--ok {
  color: var(--fd-success);
}

.fd-provider-card__notice {
  padding: 8px 10px;
  border: 1px solid var(--fd-border-subtle);
  border-radius: var(--fd-radius-control);
  background: var(--fd-bg);
}

.fd-provider-permissions {
  border-top: 1px solid var(--fd-border-subtle);
  padding-top: var(--fd-space-2);
}

.fd-provider-permissions span {
  color: var(--fd-text-secondary);
  font-size: var(--fd-text-11);
  font-weight: var(--fd-weight-semibold);
}

.fd-provider-card__footer {
  display: flex;
  align-items: center;
  gap: var(--fd-space-2);
  margin-top: auto;
  padding-top: var(--fd-space-2);
  border-top: 1px solid var(--fd-border-subtle);
}

.fd-provider-muted {
  color: var(--fd-text-muted);
  font-size: var(--fd-text-11);
}

@media (max-width: 1199px) {
  .fd-integrations-content {
    grid-template-columns: minmax(0, 1fr);
  }

  .fd-provider-card--primary {
    grid-row: auto;
  }
}
</style>
