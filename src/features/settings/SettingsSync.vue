<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRouter } from 'vue-router'

import FdBadge from '@/components/ui/FdBadge.vue'
import FdButton from '@/components/ui/FdButton.vue'
import { useApiMonitorStore } from '@/stores/apiMonitor'
import { useGithubAuthStore } from '@/stores/githubAuth'

import SettingsRow from './SettingsRow.vue'
import SettingsSection from './SettingsSection.vue'

const api = useApiMonitorStore()
const auth = useGithubAuthStore()
const router = useRouter()
const { t } = useI18n()
const refreshing = ref(false)

const connectionLabel = computed(() => {
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
    await Promise.all([api.load(), auth.loadConnection()])
  } finally {
    refreshing.value = false
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

    <div class="fd-sync-actions">
      <FdButton variant="secondary" :loading="refreshing" @click="refresh">
        {{ t('settings.sync.refresh') }}
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

.fd-sync-actions {
  display: flex;
  flex-wrap: wrap;
  gap: var(--fd-space-2);
  padding-top: var(--fd-space-3);
}
</style>
