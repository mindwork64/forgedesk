<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'

import FdBadge from '@/components/ui/FdBadge.vue'
import FdButton from '@/components/ui/FdButton.vue'
import FdKbd from '@/components/ui/FdKbd.vue'
import { useUiStore } from '@/stores/ui'

import SettingsSection from './SettingsSection.vue'

const { t } = useI18n()
const ui = useUiStore()

const rows = computed(() => [
  {
    keys: ['Ctrl', 'K'],
    label: t('settings.shortcuts.commandPalette'),
    implemented: true,
  },
  { keys: ['G', 'I'], label: t('settings.shortcuts.goInbox'), implemented: true },
  {
    keys: ['G', 'M'],
    label: t('settings.shortcuts.goMyIssues'),
    implemented: true,
  },
  {
    keys: ['G', 'P'],
    label: t('settings.shortcuts.goProjects'),
    implemented: true,
  },
  {
    keys: ['G', 'R'],
    label: t('settings.shortcuts.goRepositories'),
    implemented: true,
  },
  { keys: ['Esc'], label: t('settings.shortcuts.escape'), implemented: true },
  { keys: ['?'], label: t('settings.shortcuts.help'), implemented: true },
  {
    keys: ['C'],
    label: t('settings.shortcuts.createIssue'),
    implemented: false,
  },
  {
    keys: ['Ctrl', 'Enter'],
    label: t('settings.shortcuts.submit'),
    implemented: false,
  },
  {
    keys: ['Space'],
    label: t('settings.shortcuts.space'),
    implemented: false,
  },
])
</script>

<template>
  <SettingsSection
    :title="t('settings.shortcuts.title')"
    :description="t('settings.shortcuts.description')"
  >
    <template #actions>
      <FdButton variant="secondary" size="sm" @click="ui.openShortcuts()">
        {{ t('settings.shortcuts.help') }}
      </FdButton>
    </template>

    <div class="fd-shortcut-list">
      <div v-for="row in rows" :key="row.label" class="fd-shortcut-row">
        <span class="fd-shortcut-row__keys">
          <FdKbd :keys="row.keys" />
        </span>
        <span class="fd-shortcut-row__label">{{ row.label }}</span>
        <FdBadge :tone="row.implemented ? 'success' : 'neutral'">
          {{
            row.implemented
              ? t('settings.shortcuts.implemented')
              : t('settings.shortcuts.planned')
          }}
        </FdBadge>
      </div>
    </div>
  </SettingsSection>
</template>

<style scoped>
.fd-shortcut-list {
  display: flex;
  flex-direction: column;
}

.fd-shortcut-row {
  display: flex;
  align-items: center;
  gap: var(--fd-space-3);
  min-height: 38px;
  border-bottom: 1px solid var(--fd-border-subtle);
}

.fd-shortcut-row:last-child {
  border-bottom: 0;
}

.fd-shortcut-row__keys {
  display: inline-flex;
  flex: 0 0 132px;
}

.fd-shortcut-row__label {
  flex: 1;
  font-size: var(--fd-text-13);
}
</style>
