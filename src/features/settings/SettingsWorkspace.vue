<script setup lang="ts">
import { ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'

import FdBadge from '@/components/ui/FdBadge.vue'
import FdButton from '@/components/ui/FdButton.vue'
import FdInput from '@/components/ui/FdInput.vue'
import { useAppStore } from '@/stores/app'

import SettingsRow from './SettingsRow.vue'
import SettingsSection from './SettingsSection.vue'

const app = useAppStore()
const { t } = useI18n()
const keyPrefix = ref(app.workspace?.keyPrefix ?? 'FD')
const savingPrefix = ref(false)

watch(
  () => app.workspace?.keyPrefix,
  (value) => {
    if (value) keyPrefix.value = value
  },
)

async function saveKeyPrefix(): Promise<void> {
  if (savingPrefix.value) return
  savingPrefix.value = true
  const saved = await app.updateWorkspaceKeyPrefix(keyPrefix.value)
  if (saved) keyPrefix.value = app.workspace?.keyPrefix ?? keyPrefix.value.trim().toUpperCase()
  savingPrefix.value = false
}
</script>

<template>
  <SettingsSection
    :title="t('settings.workspace.title')"
    :description="t('settings.workspace.description')"
  >
    <SettingsRow :label="t('settings.workspace.name')">
      <span class="fd-settings-value">{{ app.workspace?.name ?? '—' }}</span>
    </SettingsRow>

    <SettingsRow :label="t('settings.workspace.id')">
      <span class="fd-settings-mono">{{ app.workspace?.id ?? '—' }}</span>
    </SettingsRow>

    <SettingsRow
      :label="t('settings.workspace.keyPrefix')"
      :hint="t('settings.workspace.keyPrefixHint')"
      stacked
    >
      <form class="fd-workspace-prefix" @submit.prevent="saveKeyPrefix">
        <FdInput
          v-model="keyPrefix"
          class="fd-workspace-prefix__input"
          maxlength="10"
          autocomplete="off"
          spellcheck="false"
          :placeholder="t('settings.workspace.keyPrefixPlaceholder')"
          :aria-label="t('settings.workspace.keyPrefix')"
        />
        <FdButton
          type="submit"
          variant="secondary"
          :loading="savingPrefix"
          :disabled="savingPrefix || keyPrefix.trim().toUpperCase() === app.workspace?.keyPrefix"
        >
          {{ t('settings.workspace.keyPrefixSave') }}
        </FdButton>
      </form>
    </SettingsRow>

    <SettingsRow
      :label="t('settings.workspace.workflow')"
      :hint="t('settings.workspace.workflowHint')"
    >
      <FdBadge tone="neutral">{{ t('settings.workspace.builtIn') }}</FdBadge>
    </SettingsRow>
  </SettingsSection>
</template>

<style scoped>
.fd-workspace-prefix {
  display: flex;
  align-items: center;
  gap: var(--fd-space-2);
}
.fd-workspace-prefix__input {
  width: 160px;
}
</style>
