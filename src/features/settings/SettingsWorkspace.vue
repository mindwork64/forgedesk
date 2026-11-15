<script setup lang="ts">
import { useI18n } from 'vue-i18n'

import FdBadge from '@/components/ui/FdBadge.vue'
import { useAppStore } from '@/stores/app'

import SettingsRow from './SettingsRow.vue'
import SettingsSection from './SettingsSection.vue'

const app = useAppStore()
const { t } = useI18n()
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
    >
      <span class="fd-settings-mono">{{ app.workspace?.keyPrefix ?? 'FD' }}</span>
    </SettingsRow>

    <SettingsRow
      :label="t('settings.workspace.workflow')"
      :hint="t('settings.workspace.workflowHint')"
    >
      <FdBadge tone="neutral">{{ t('common.stage', { stage: 2 }) }}</FdBadge>
    </SettingsRow>
  </SettingsSection>
</template>
