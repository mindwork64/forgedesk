<script setup lang="ts">
import { RefreshCw } from '@lucide/vue'
import { computed, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRouter } from 'vue-router'

import FdBadge from '@/components/ui/FdBadge.vue'
import FdButton from '@/components/ui/FdButton.vue'
import { useGitStore } from '@/stores/git'

import SettingsRow from './SettingsRow.vue'
import SettingsSection from './SettingsSection.vue'

const git = useGitStore()
const router = useRouter()
const { t } = useI18n()

const checking = ref(true)

const availabilityState = computed<'checking' | 'available' | 'missing' | 'failed'>(() => {
  if (checking.value || !git.availability) return 'checking'
  if (git.availability.available) return 'available'
  return /git is not available/i.test(git.availability.error ?? '') ? 'missing' : 'failed'
})

const statusTone = computed(() => {
  switch (availabilityState.value) {
    case 'available':
      return 'success'
    case 'checking':
      return 'neutral'
    case 'missing':
      return 'warning'
    default:
      return 'danger'
  }
})

const statusLabel = computed(() => {
  switch (availabilityState.value) {
    case 'available':
      return t('settings.git.available')
    case 'checking':
      return t('settings.git.checking')
    case 'missing':
      return t('settings.git.missing')
    default:
      return t('settings.git.failed')
  }
})

async function refresh(): Promise<void> {
  checking.value = true
  await Promise.all([git.loadAvailability(), git.loadRepositories()])
  checking.value = false
}

onMounted(refresh)
</script>

<template>
  <SettingsSection :title="t('settings.git.title')" :description="t('settings.git.description')">
    <SettingsRow :label="t('settings.git.status')">
      <FdBadge :tone="statusTone">{{ statusLabel }}</FdBadge>
    </SettingsRow>

    <SettingsRow v-if="availabilityState === 'available'" :label="t('settings.git.version')">
      <span class="fd-mono">{{ git.availability?.version ?? '—' }}</span>
    </SettingsRow>

    <p v-if="availabilityState === 'missing'" class="fd-git-note">
      {{ t('settings.git.missingHint') }}
    </p>
    <p v-else-if="availabilityState === 'failed'" class="fd-git-note fd-git-note--error">
      {{ t('settings.git.failedHint', { error: git.availability?.error ?? '' }) }}
    </p>

    <SettingsRow
      :label="t('settings.git.repositories')"
      :hint="t('settings.git.repositoriesHint')"
    >
      <span class="fd-git-count">{{ git.repositories.length }}</span>
    </SettingsRow>
    <p v-if="git.error" class="fd-git-note fd-git-note--error">{{ git.error.message }}</p>

    <SettingsRow :label="t('settings.git.operations')" stacked>
      <p class="fd-git-operations">{{ t('settings.git.operationsHint') }}</p>
    </SettingsRow>

    <div class="fd-git-actions">
      <FdButton variant="secondary" :loading="checking" @click="refresh">
        <RefreshCw :size="14" aria-hidden="true" />
        {{ t('settings.git.refresh') }}
      </FdButton>
      <FdButton variant="primary" @click="router.push('/repositories')">
        {{ t('settings.git.manage') }}
      </FdButton>
    </div>
  </SettingsSection>
</template>

<style scoped>
.fd-git-count {
  color: var(--fd-text-primary);
  font-size: var(--fd-text-13);
  font-variant-numeric: tabular-nums;
}

.fd-git-note {
  margin: 0 0 8px;
  color: var(--fd-text-muted);
  font-size: var(--fd-text-12);
  line-height: var(--fd-line-body);
}

.fd-git-note--error {
  color: var(--fd-danger);
}

.fd-git-operations {
  max-width: 620px;
  margin: 0;
  color: var(--fd-text-secondary);
  font-size: var(--fd-text-12);
  line-height: var(--fd-line-body);
}

.fd-git-actions {
  display: flex;
  flex-wrap: wrap;
  gap: var(--fd-space-2);
  padding-top: 12px;
}
</style>
