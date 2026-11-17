<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRouter } from 'vue-router'

import FdBadge from '@/components/ui/FdBadge.vue'
import FdButton from '@/components/ui/FdButton.vue'
import { useApiMonitorStore } from '@/stores/apiMonitor'
import { useGithubAuthStore } from '@/stores/githubAuth'
import { useToastStore } from '@/stores/toasts'
import { api as ipc, toIpcError } from '@/lib/ipc'
import type { SyncConflict } from '@/types/ipc'

import SettingsRow from './SettingsRow.vue'
import SettingsSection from './SettingsSection.vue'

const api = useApiMonitorStore()
const auth = useGithubAuthStore()
const toast = useToastStore()
const conflicts = ref<SyncConflict[]>([])
const resolvingConflict = ref<string | null>(null)
const router = useRouter()
const { t } = useI18n()
const refreshing = ref(false)

const connectionLabel = computed(() => {
  if (auth.connectionLoading || !auth.connectionChecked) return t('status.githubChecking')
  if (auth.connected) {
    return t('settings.sync.connected', { login: auth.connection?.login ?? '' })
  }
  if (auth.storageUnavailable) return t('settings.sync.credentialUnavailable')
  if (auth.connectionCheckFailed) return t('settings.sync.connectionCheckFailed')
  return t('settings.sync.disconnected')
})

const connectionTone = computed<'success' | 'warning' | 'neutral'>(() => {
  if (auth.connected) return 'success'
  if (auth.storageUnavailable || auth.connectionCheckFailed) return 'warning'
  return 'neutral'
})

const apiSummary = computed(() => {
  if (api.loading && !api.state) return t('settings.sync.apiLoading')
  if (api.error) return api.error.message || t('settings.sync.apiError')
  const state = api.state
  return t('settings.sync.apiSummary', {
    buckets: state?.budgets.length ?? 0,
    requests: state?.requests.length ?? 0,
    queued: state?.queue.length ?? 0,
  })
})

async function refresh(): Promise<void> {
  refreshing.value = true
  try {
    await Promise.all([api.load(), auth.loadConnection(true)])
    conflicts.value = await ipc.githubSyncConflicts()
  } finally {
    refreshing.value = false
  }
}

async function pushPending(): Promise<void> {
  const selected = auth.selectedRepo
  if (!selected) return
  await auth.dispatchIssueOutbox(selected.owner, selected.name)
}

async function resolveConflict(conflict: SyncConflict, resolution: 'local' | 'remote'): Promise<void> {
  resolvingConflict.value = conflict.id
  try {
    await ipc.githubResolveSyncConflict(conflict.id, resolution)
    conflicts.value = await ipc.githubSyncConflicts()
    toast.success(t('settings.sync.conflictResolved'))
  } catch (caught) {
    const wire = toIpcError(caught)
    toast.error(t('settings.sync.conflictResolveFailed'), wire.message, wire.detail)
  } finally {
    resolvingConflict.value = null
  }
}

onMounted(() => {
  void refresh()
})
</script>

<template>
  <SettingsSection
    :title="t('settings.sync.title')"
    :description="t('settings.sync.description')"
  >
    <SettingsRow :label="t('settings.sync.connection')">
      <FdBadge :tone="connectionTone">{{ connectionLabel }}</FdBadge>
    </SettingsRow>

    <SettingsRow :label="t('settings.sync.mode')">
      <FdBadge tone="warning">{{ t('settings.sync.manualOnly') }}</FdBadge>
    </SettingsRow>

    <SettingsRow :label="t('settings.sync.background')" stacked>
      <div class="fd-sync-status">
        <FdBadge tone="warning">{{ t('settings.sync.backgroundUnavailable') }}</FdBadge>
        <p>{{ t('settings.sync.backgroundBody') }}</p>
      </div>
    </SettingsRow>

    <SettingsRow :label="t('settings.sync.availableNow')" stacked>
      <p class="fd-sync-note">{{ t('settings.sync.currentCapabilities') }}</p>
    </SettingsRow>

    <SettingsRow :label="t('settings.sync.apiSnapshot')" stacked>
      <p class="fd-sync-note fd-sync-note--mono">{{ apiSummary }}</p>
      <p class="fd-sync-note">{{ t('settings.sync.apiLimitations') }}</p>
    </SettingsRow>

    <SettingsRow v-if="conflicts.length > 0" :label="t('settings.sync.conflicts')" stacked>
      <div class="fd-sync-conflicts">
        <div v-for="conflict in conflicts" :key="conflict.id" class="fd-sync-conflict">
          <div class="fd-sync-conflict__details">
            <strong>{{ conflict.field }} · {{ conflict.localIssueId }}</strong>
            <small>{{ t('settings.sync.localValue') }}: {{ conflict.localValue ?? '—' }}</small>
            <small>{{ t('settings.sync.remoteValue') }}: {{ conflict.remoteValue ?? '—' }}</small>
          </div>
          <div class="fd-sync-conflict__actions">
            <FdButton
              size="sm"
              variant="secondary"
              :loading="resolvingConflict === conflict.id"
              :disabled="resolvingConflict !== null"
              @click="resolveConflict(conflict, 'local')"
            >
              {{ t('settings.sync.keepLocal') }}
            </FdButton>
            <FdButton
              size="sm"
              variant="primary"
              :loading="resolvingConflict === conflict.id"
              :disabled="resolvingConflict !== null"
              @click="resolveConflict(conflict, 'remote')"
            >
              {{ t('settings.sync.useGitHub') }}
            </FdButton>
          </div>
        </div>
      </div>
    </SettingsRow>

    <div class="fd-sync-actions">
      <FdButton variant="secondary" :loading="refreshing" @click="refresh">
        {{ t('settings.sync.refresh') }}
      </FdButton>
      <FdButton
        v-if="auth.connected && auth.selectedRepo"
        variant="secondary"
        :loading="auth.syncingOutbox"
        @click="pushPending"
      >
        {{ t('settings.sync.pushPending') }}
      </FdButton>
      <FdButton variant="secondary" @click="router.push('/api-monitor')">
        {{ t('settings.sync.openApiMonitor') }}
      </FdButton>
      <FdButton variant="primary" @click="router.push('/integrations')">
        {{ t('settings.sync.openIntegrations') }}
      </FdButton>
    </div>
  </SettingsSection>
</template>

<style scoped>
.fd-sync-status {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: var(--fd-space-2);
}

.fd-sync-status p,
.fd-sync-note {
  max-width: 620px;
  margin: 0;
  color: var(--fd-text-secondary);
  font-size: var(--fd-text-12);
  line-height: var(--fd-line-body);
}

.fd-sync-note--mono {
  color: var(--fd-text-primary);
  font-variant-numeric: tabular-nums;
}

.fd-sync-note + .fd-sync-note {
  margin-top: var(--fd-space-2);
}

.fd-sync-conflicts {
  display: flex;
  flex-direction: column;
  gap: var(--fd-space-3);
  width: 100%;
}

.fd-sync-conflict {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--fd-space-3);
  padding: var(--fd-space-3);
  border: 1px solid var(--fd-border-subtle);
  border-radius: var(--fd-radius-md);
}

.fd-sync-conflict__details {
  display: flex;
  flex-direction: column;
  min-width: 0;
  gap: 3px;
}

.fd-sync-conflict__details strong {
  font-size: var(--fd-text-12);
}

.fd-sync-conflict__details small {
  overflow-wrap: anywhere;
  color: var(--fd-text-muted);
  font-size: var(--fd-text-11);
}

.fd-sync-conflict__actions {
  display: flex;
  flex: none;
  gap: var(--fd-space-2);
}

.fd-sync-actions {
  display: flex;
  flex-wrap: wrap;
  gap: var(--fd-space-2);
  padding-top: var(--fd-space-3);
}
</style>
