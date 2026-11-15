<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'

import FdSelect from '@/components/ui/FdSelect.vue'
import { useAppStore } from '@/stores/app'
import type { LocalePreference } from '@/types/ipc'

import SettingsRow from './SettingsRow.vue'
import SettingsSection from './SettingsSection.vue'

const app = useAppStore()
const { t } = useI18n()

const locale = computed(() => app.settings?.locale ?? 'system')
const timeZone = Intl.DateTimeFormat().resolvedOptions().timeZone ?? '—'

function updateLocale(value: string): void {
  void app.setLocale(value as LocalePreference)
}
</script>

<template>
  <SettingsSection
    :title="t('settings.general.title')"
    :description="t('settings.general.description')"
  >
    <SettingsRow
      :label="t('settings.general.language')"
      :hint="t('settings.general.languageHint')"
    >
      <div class="fd-settings-control">
        <FdSelect
          :model-value="locale"
          :aria-label="t('settings.general.language')"
          @update:model-value="updateLocale"
        >
          <option value="system">{{ t('common.system') }}</option>
          <option value="en">English</option>
          <option value="ru">Русский</option>
        </FdSelect>
      </div>
    </SettingsRow>

    <SettingsRow
      :label="t('settings.general.timezone')"
      :hint="t('settings.general.timezoneHint')"
    >
      <span class="fd-settings-value fd-mono">{{ timeZone }}</span>
    </SettingsRow>

    <SettingsRow
      :label="t('settings.general.startup')"
      :hint="t('settings.general.startupHint')"
    >
      <span class="fd-settings-value">{{ t('settings.general.restoreLastSession') }}</span>
    </SettingsRow>

    <SettingsRow :label="t('settings.general.build')">
      <span class="fd-settings-value fd-mono">
        v{{ app.appVersion }} · schema {{ app.schemaVersion }}
      </span>
    </SettingsRow>
  </SettingsSection>
</template>

<style scoped>
.fd-settings-control {
  width: 200px;
}
</style>
