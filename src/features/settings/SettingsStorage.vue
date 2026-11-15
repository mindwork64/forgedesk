<script setup lang="ts">
import { Copy, Plus } from '@lucide/vue'
import { computed, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'

import FdBadge from '@/components/ui/FdBadge.vue'
import FdButton from '@/components/ui/FdButton.vue'
import FdConfirmDialog from '@/components/ui/FdConfirmDialog.vue'
import FdIconButton from '@/components/ui/FdIconButton.vue'
import FdInput from '@/components/ui/FdInput.vue'
import FdSwitch from '@/components/ui/FdSwitch.vue'
import { copyText } from '@/lib/clipboard'
import { formatBytes, formatDateTime } from '@/lib/format'
import { useAppStore } from '@/stores/app'
import { useToastStore } from '@/stores/toasts'
import type { BackupInfo } from '@/types/ipc'

import SettingsRow from './SettingsRow.vue'
import SettingsSection from './SettingsSection.vue'

const app = useAppStore()
const toast = useToastStore()
const { t } = useI18n()

const busy = ref(false)
const restoreTarget = ref<BackupInfo | null>(null)
const restoreOpen = ref(false)
const retentionDraft = ref(String(app.settings?.backupRetention ?? 20))
const retentionError = ref<string | null>(null)

const restoreBody = computed(() =>
  t('settings.storage.restoreBody', {
    date: restoreTarget.value
      ? formatDateTime(restoreTarget.value.createdAt, app.resolvedLocale)
      : '',
  }),
)

watch(
  () => app.settings?.backupRetention,
  (value) => {
    retentionDraft.value = String(value ?? 20)
  },
)

async function copy(value: string | undefined): Promise<void> {
  const ok = await copyText(value ?? '')
  if (ok) {
    toast.success(t('toasts.copySuccess'))
  } else {
    toast.error(t('toasts.copyError'))
  }
}

async function createBackup(): Promise<void> {
  if (busy.value) {
    return
  }
  busy.value = true
  await app.createBackup()
  busy.value = false
}

function askRestore(backup: BackupInfo): void {
  if (!backup.valid) {
    return
  }
  restoreTarget.value = backup
  restoreOpen.value = true
}

async function confirmRestore(): Promise<void> {
  const target = restoreTarget.value
  if (!target || busy.value) {
    return
  }
  busy.value = true
  const result = await app.restoreBackup(target.id)
  busy.value = false
  if (result) {
    restoreOpen.value = false
    restoreTarget.value = null
  }
}

function onAutoBackup(value: boolean): void {
  void app.setAutoBackupEnabled(value)
}

async function applyRetention(): Promise<void> {
  const parsed = Number.parseInt(retentionDraft.value, 10)
  if (!Number.isFinite(parsed) || parsed < 3 || parsed > 100) {
    retentionError.value = t('settings.storage.retentionError')
    retentionDraft.value = String(app.settings?.backupRetention ?? 20)
    return
  }
  retentionError.value = null
  const ok = await app.setBackupRetention(parsed)
  if (!ok) {
    retentionDraft.value = String(app.settings?.backupRetention ?? 20)
  }
}
</script>

<template>
  <div class="fd-storage">
    <SettingsSection
      :title="t('settings.storage.title')"
      :description="t('settings.storage.description')"
    >
      <SettingsRow :label="t('settings.storage.dbPath')" stacked>
        <div class="fd-storage__path">
          <span class="fd-settings-mono">{{ app.data?.dbPath ?? '—' }}</span>
          <FdIconButton
            :label="t('settings.storage.copyPath')"
            @click="copy(app.data?.dbPath)"
          >
            <Copy :size="12" />
          </FdIconButton>
        </div>
      </SettingsRow>

      <SettingsRow :label="t('settings.storage.dataDir')" stacked>
        <div class="fd-storage__path">
          <span class="fd-settings-mono">{{ app.data?.dataDir ?? '—' }}</span>
          <FdIconButton
            :label="t('settings.storage.copyPath')"
            @click="copy(app.data?.dataDir)"
          >
            <Copy :size="12" />
          </FdIconButton>
        </div>
      </SettingsRow>

      <SettingsRow :label="t('settings.storage.backupsDir')" stacked>
        <div class="fd-storage__path">
          <span class="fd-settings-mono">{{ app.data?.backupsDir ?? '—' }}</span>
          <FdIconButton
            :label="t('settings.storage.copyPath')"
            @click="copy(app.data?.backupsDir)"
          >
            <Copy :size="12" />
          </FdIconButton>
        </div>
      </SettingsRow>

      <SettingsRow :label="t('settings.storage.schema')">
        <span class="fd-settings-value fd-mono">v{{ app.schemaVersion }}</span>
      </SettingsRow>

      <SettingsRow
        :label="t('settings.storage.autoBackup')"
        :hint="t('settings.storage.autoBackupHint')"
      >
        <FdSwitch
          :model-value="app.settings?.autoBackupEnabled ?? true"
          :label="t('settings.storage.autoBackup')"
          @update:model-value="onAutoBackup"
        />
      </SettingsRow>

      <SettingsRow
        :label="t('settings.storage.retention')"
        :hint="t('settings.storage.retentionHint')"
      >
        <div class="fd-storage__retention-wrap">
          <div class="fd-storage__retention">
            <FdInput
              v-model="retentionDraft"
              type="number"
              :invalid="!!retentionError"
              :aria-label="t('settings.storage.retention')"
              class="fd-storage__retention-input"
              @update:model-value="retentionError = null"
              @blur="applyRetention"
              @keydown.enter="applyRetention"
            />
            <span class="fd-settings-value">{{ t('settings.storage.snapshotsSuffix') }}</span>
          </div>
          <p v-if="retentionError" class="fd-storage__error" role="alert">
            {{ retentionError }}
          </p>
        </div>
      </SettingsRow>
    </SettingsSection>

    <SettingsSection
      :title="t('settings.storage.backupList')"
      :description="t('settings.storage.backupListDescription')"
    >
      <template #actions>
        <FdButton variant="primary" size="sm" :loading="busy" @click="createBackup">
          <Plus :size="13" aria-hidden="true" />
          {{ t('settings.storage.createBackup') }}
        </FdButton>
      </template>

      <div
        v-if="app.backups.length > 0"
        class="fd-storage__table"
        data-testid="backup-table"
        role="table"
        :aria-label="t('settings.storage.backupList')"
      >
        <div class="fd-storage__row fd-storage__row--head" role="row">
          <span role="columnheader">{{ t('settings.storage.createdAt') }}</span>
          <span role="columnheader">{{ t('settings.storage.size') }}</span>
          <span role="columnheader">{{ t('settings.storage.schemaShort') }}</span>
          <span role="columnheader">{{ t('settings.storage.status') }}</span>
          <span class="fd-visually-hidden" role="columnheader">{{ t('common.actions') }}</span>
        </div>
        <div
          v-for="backup in app.backups"
          :key="backup.id"
          class="fd-storage__row"
          role="row"
        >
          <span class="fd-storage__date" role="cell">
            {{ formatDateTime(backup.createdAt, app.resolvedLocale) }}
          </span>
          <span class="fd-settings-value" role="cell">
            {{ formatBytes(backup.sizeBytes, app.resolvedLocale) }}
          </span>
          <span class="fd-settings-value fd-mono" role="cell">v{{ backup.schemaVersion }}</span>
          <span role="cell">
            <FdBadge :tone="backup.valid ? 'success' : 'danger'">
              {{
                backup.valid
                  ? t('settings.storage.valid')
                  : t('settings.storage.invalid')
              }}
            </FdBadge>
          </span>
          <span class="fd-storage__actions" role="cell">
            <FdButton
              variant="secondary"
              size="sm"
              :disabled="!backup.valid || busy"
              @click="askRestore(backup)"
            >
              {{ t('settings.storage.restore') }}
            </FdButton>
          </span>
        </div>
      </div>

      <div v-else class="fd-storage__empty">
        <p class="fd-storage__empty-title">{{ t('settings.storage.noBackups') }}</p>
        <p class="fd-storage__empty-hint">{{ t('settings.storage.noBackupsHint') }}</p>
      </div>

      <p class="fd-settings-note">{{ t('settings.storage.restoreSafety') }}</p>
      <p class="fd-settings-note">{{ t('settings.storage.externalImport') }}</p>
    </SettingsSection>

    <FdConfirmDialog
      v-model="restoreOpen"
      :title="t('settings.storage.restoreTitle')"
      :body="restoreBody"
      :confirm-label="t('settings.storage.restoreConfirm')"
      destructive
      :busy="busy"
      @confirm="confirmRestore"
    />
  </div>
</template>

<style scoped>
.fd-storage {
  display: flex;
  flex-direction: column;
  gap: var(--fd-space-7);
}

.fd-storage__path {
  display: flex;
  align-items: flex-start;
  gap: var(--fd-space-2);
  width: 100%;
}

.fd-storage__path span {
  flex: 1;
  min-width: 0;
}

.fd-storage__retention-wrap {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
}

.fd-storage__retention {
  display: flex;
  align-items: center;
  gap: var(--fd-space-2);
}

.fd-storage__retention-input {
  width: 88px;
}

.fd-storage__error {
  margin-top: 4px;
  color: var(--fd-danger);
  font-size: var(--fd-text-11);
}

.fd-storage__table {
  display: flex;
  flex-direction: column;
  border-top: 1px solid var(--fd-border-subtle);
}

.fd-storage__row {
  display: grid;
  grid-template-columns: minmax(150px, 1fr) 78px 60px 80px 86px;
  gap: var(--fd-space-3);
  align-items: center;
  min-height: var(--fd-row-height);
  border-bottom: 1px solid var(--fd-border-subtle);
}

.fd-storage__row--head {
  min-height: 32px;
  color: var(--fd-text-muted);
  font-size: var(--fd-text-12);
}

.fd-storage__date {
  color: var(--fd-text);
  font-size: var(--fd-text-12);
  white-space: nowrap;
}

.fd-storage__actions {
  display: flex;
  justify-content: flex-end;
}

.fd-storage__empty {
  padding: var(--fd-space-5) 0;
}

.fd-storage__empty-title {
  font-size: var(--fd-text-13);
  font-weight: var(--fd-weight-medium);
}

.fd-storage__empty-hint {
  margin-top: 2px;
  color: var(--fd-text-muted);
  font-size: var(--fd-text-12);
}

.fd-settings-note {
  margin-top: var(--fd-space-3);
}
</style>
