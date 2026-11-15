<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'

import FdSegmented from '@/components/ui/FdSegmented.vue'
import { useAppStore } from '@/stores/app'
import type { ThemePreference } from '@/types/ipc'

import SettingsRow from './SettingsRow.vue'
import SettingsSection from './SettingsSection.vue'

const app = useAppStore()
const { t } = useI18n()

const theme = computed<string>(() => app.settings?.theme ?? 'dark')
const themeOptions = computed(() => [
  { value: 'dark', label: t('settings.appearance.themeDark') },
  { value: 'light', label: t('settings.appearance.themeLight') },
  { value: 'system', label: t('settings.appearance.themeSystem') },
])

function updateTheme(value: string): void {
  void app.setTheme(value as ThemePreference)
}
</script>

<template>
  <SettingsSection
    :title="t('settings.appearance.title')"
    :description="t('settings.appearance.description')"
  >
    <SettingsRow
      :label="t('settings.appearance.theme')"
      :hint="t('settings.appearance.themeHint')"
    >
      <FdSegmented
        :model-value="theme"
        :options="themeOptions"
        :label="t('settings.appearance.theme')"
        @update:model-value="updateTheme"
      />
    </SettingsRow>

    <SettingsRow
      :label="t('settings.appearance.density')"
      :hint="t('settings.appearance.densityHint')"
    >
      <span class="fd-settings-value">{{ t('settings.appearance.densityStandard') }}</span>
    </SettingsRow>
  </SettingsSection>
</template>
