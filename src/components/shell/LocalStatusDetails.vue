<script setup lang="ts">
import { Copy } from '@lucide/vue'
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'

import { copyText } from '@/lib/clipboard'
import { formatDateTime, formatRelative } from '@/lib/format'
import { useAppStore } from '@/stores/app'
import { useGithubAuthStore } from '@/stores/githubAuth'
import { useToastStore } from '@/stores/toasts'

import FdBadge from '../ui/FdBadge.vue'
import FdIconButton from '../ui/FdIconButton.vue'

const app = useAppStore()
const auth = useGithubAuthStore()
const toast = useToastStore()
const { t } = useI18n()

const lastBackup = computed(() => app.backups[0] ?? null)
const githubStatus = computed(() => {
  if (auth.connectionLoading || !auth.connectionChecked) return t('status.githubChecking')
  if (auth.connected) return t('status.githubConnected', { login: auth.connection?.login ?? '' })
  if (auth.storageUnavailable) return t('status.githubStorageUnavailable')
  if (auth.connectionCheckFailed) return t('status.githubCheckFailed')
  return t('status.githubDisconnected')
})
const githubStatusNote = computed(() => {
  if (auth.connectionLoading || !auth.connectionChecked) return t('status.githubChecking')
  if (auth.storageUnavailable) return t('githubAuth.storageUnavailableHint')
  if (auth.connectionCheckFailed) return t('githubAuth.connectionCheckFailedHint')
  return auth.connected ? t('status.githubConnectedNote') : t('status.githubDisconnectedNote')
})

async function copy(value: string): Promise<void> {
  const ok = await copyText(value)
  if (ok) {
    toast.success(t('toasts.copySuccess'))
  } else {
    toast.error(t('toasts.copyError'))
  }
}
</script>

<template>
  <div class="fd-status-details">
    <div class="fd-status-details__heading">
      <span class="fd-status-details__dot" aria-hidden="true" />
      <span>{{ t('status.localReady') }}</span>
      <FdBadge tone="accent">{{ t('status.localStorage') }}</FdBadge>
    </div>

    <dl class="fd-status-details__grid">
      <dt>{{ t('integrations.github') }}</dt>
      <dd>{{ githubStatus }}</dd>
      <dt>{{ t('status.schema') }}</dt>
      <dd class="fd-mono">v{{ app.schemaVersion }}</dd>
      <dt>{{ t('status.version') }}</dt>
      <dd class="fd-mono">{{ app.appVersion }}</dd>
      <dt>{{ t('status.dbPath') }}</dt>
      <dd class="fd-status-details__path">
        <span class="fd-mono" :title="app.data?.dbPath">{{ app.data?.dbPath }}</span>
        <FdIconButton
          :label="t('settings.storage.copyPath')"
          @click="copy(app.data?.dbPath ?? '')"
        >
          <Copy :size="12" />
        </FdIconButton>
      </dd>
      <dt>{{ t('status.dataDir') }}</dt>
      <dd class="fd-status-details__path">
        <span class="fd-mono" :title="app.data?.dataDir">{{ app.data?.dataDir }}</span>
        <FdIconButton
          :label="t('settings.storage.copyPath')"
          @click="copy(app.data?.dataDir ?? '')"
        >
          <Copy :size="12" />
        </FdIconButton>
      </dd>
      <dt>{{ t('status.backups') }}</dt>
      <dd>
        <template v-if="lastBackup">
          <span :title="formatDateTime(lastBackup.createdAt, app.resolvedLocale)">
            {{ formatRelative(lastBackup.createdAt, app.resolvedLocale) }}
          </span>
        </template>
        <span v-else>{{ t('status.never') }}</span>
      </dd>
      <dt>{{ t('status.autoBackup') }}</dt>
      <dd>
        {{ app.settings?.autoBackupEnabled ? t('status.on') : t('status.off') }}
      </dd>
    </dl>

    <p class="fd-status-details__note">{{ githubStatusNote }}</p>
  </div>
</template>

<style scoped>
.fd-status-details {
  display: flex;
  flex-direction: column;
  gap: var(--fd-space-2);
  padding: 2px;
}

.fd-status-details__heading {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 2px 4px 6px;
  font-size: var(--fd-text-12);
  font-weight: var(--fd-weight-medium);
}

.fd-status-details__dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--fd-success);
}

.fd-status-details__grid {
  display: grid;
  grid-template-columns: 96px minmax(0, 1fr);
  gap: 6px 10px;
  margin: 0;
  padding: 8px;
  border: 1px solid var(--fd-border-subtle);
  border-radius: var(--fd-radius-control);
  background: var(--fd-surface);
}

.fd-status-details__grid dt {
  color: var(--fd-text-muted);
  font-size: var(--fd-text-11);
}

.fd-status-details__grid dd {
  min-width: 0;
  margin: 0;
  color: var(--fd-text-secondary);
  font-size: var(--fd-text-11);
  overflow-wrap: anywhere;
}

.fd-status-details__path {
  display: flex;
  align-items: flex-start;
  gap: 4px;
}

.fd-status-details__path span {
  flex: 1;
  min-width: 0;
  overflow-wrap: anywhere;
}

.fd-status-details__note {
  padding: 0 4px;
  color: var(--fd-text-muted);
  font-size: var(--fd-text-11);
  line-height: var(--fd-line-body);
}
</style>
