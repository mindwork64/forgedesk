<script setup lang="ts">
import { Copy } from '@lucide/vue'
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'

import FdButton from '@/components/ui/FdButton.vue'
import { copyText } from '@/lib/clipboard'
import { useAppStore } from '@/stores/app'
import { useToastStore } from '@/stores/toasts'

import SettingsSection from './SettingsSection.vue'

const app = useAppStore()
const toast = useToastStore()
const { t } = useI18n()

const diagnostics = computed(() =>
  JSON.stringify(
    {
      application: 'ForgeDesk',
      version: app.appVersion || 'unknown',
      schemaVersion: app.schemaVersion,
      databasePath: app.data?.dbPath ?? null,
      dataDirectory: app.data?.dataDir ?? null,
      backupsDirectory: app.data?.backupsDir ?? null,
      locale: app.resolvedLocale,
      theme: app.resolvedTheme,
      platform: navigator.userAgent,
      generatedAt: new Date().toISOString(),
      containsCredentials: false,
    },
    null,
    2,
  ),
)

async function copyDiagnostics(): Promise<void> {
  const ok = await copyText(diagnostics.value)
  if (ok) {
    toast.success(t('settings.advanced.copied'))
  } else {
    toast.error(t('settings.advanced.copyError'))
  }
}
</script>

<template>
  <SettingsSection
    :title="t('settings.advanced.title')"
    :description="t('settings.advanced.description')"
  >
    <div class="fd-settings-advanced">
      <div class="fd-settings-advanced__header">
        <div>
          <p class="fd-settings-advanced__label">{{ t('settings.advanced.diagnostics') }}</p>
          <p class="fd-settings-advanced__hint">{{ t('settings.advanced.diagnosticsHint') }}</p>
        </div>
        <FdButton variant="secondary" size="sm" @click="copyDiagnostics">
          <Copy :size="13" aria-hidden="true" />
          {{ t('settings.advanced.copyDiagnostics') }}
        </FdButton>
      </div>
      <pre class="fd-settings-advanced__pre">{{ diagnostics }}</pre>

      <div class="fd-settings-note">
        <strong>{{ t('settings.advanced.reset') }}.</strong>
        {{ t('settings.advanced.resetHint') }}
      </div>
    </div>
  </SettingsSection>
</template>

<style scoped>
.fd-settings-advanced {
  display: flex;
  flex-direction: column;
  gap: var(--fd-space-3);
}

.fd-settings-advanced__header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: var(--fd-space-4);
}

.fd-settings-advanced__label {
  font-size: var(--fd-text-13);
  font-weight: var(--fd-weight-medium);
}

.fd-settings-advanced__hint {
  max-width: 440px;
  margin-top: 2px;
  color: var(--fd-text-muted);
  font-size: var(--fd-text-12);
  line-height: var(--fd-line-body);
}

.fd-settings-advanced__pre {
  max-height: 300px;
  margin: 0;
  padding: var(--fd-space-3);
  overflow: auto;
  border: 1px solid var(--fd-border);
  border-radius: var(--fd-radius-control);
  background: var(--fd-surface);
  color: var(--fd-text-secondary);
  font-family: var(--fd-font-mono);
  font-size: var(--fd-text-11);
  line-height: var(--fd-line-body);
  white-space: pre-wrap;
  word-break: break-word;
}
</style>
