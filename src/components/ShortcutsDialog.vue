<script setup lang="ts">
import { Check } from '@lucide/vue'
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'

import FdButton from '@/components/ui/FdButton.vue'
import FdKbd from '@/components/ui/FdKbd.vue'
import FdModal from '@/components/ui/FdModal.vue'
import { useUiStore } from '@/stores/ui'

interface ShortcutRow {
  keys: string[]
  label: string
  implemented: boolean
}

const ui = useUiStore()
const { t } = useI18n()

const open = computed({
  get: () => ui.shortcutsOpen,
  set: (value: boolean) => {
    if (value) {
      ui.openShortcuts()
    } else {
      ui.closeShortcuts()
    }
  },
})

const rows = computed<ShortcutRow[]>(() => [
  {
    keys: ['Ctrl', 'K'],
    label: t('shortcuts.commandPalette'),
    implemented: true,
  },
  { keys: ['G', 'I'], label: t('shortcuts.goInbox'), implemented: true },
  { keys: ['G', 'M'], label: t('shortcuts.goMyIssues'), implemented: true },
  { keys: ['G', 'P'], label: t('shortcuts.goProjects'), implemented: true },
  {
    keys: ['G', 'R'],
    label: t('shortcuts.goRepositories'),
    implemented: true,
  },
  { keys: ['?'], label: t('shortcuts.help'), implemented: true },
  { keys: ['Esc'], label: t('shortcuts.escape'), implemented: true },
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
  <FdModal v-model="open" :title="t('shortcuts.title')" size="md">
    <div class="fd-shortcuts">
      <div
        v-for="row in rows"
        :key="row.label"
        class="fd-shortcuts__row"
      >
        <span class="fd-shortcuts__keys">
          <FdKbd :keys="row.keys" />
        </span>
        <span class="fd-shortcuts__label">{{ row.label }}</span>
        <span
          v-if="row.implemented"
          class="fd-shortcuts__status fd-shortcuts__status--ready"
        >
          <Check :size="12" aria-hidden="true" />
          {{ t('settings.shortcuts.implemented') }}
        </span>
        <span v-else class="fd-shortcuts__status">
          {{ t('shortcuts.unavailable') }}
        </span>
      </div>
    </div>
    <template #footer>
      <FdButton variant="secondary" @click="open = false">
        {{ t('shortcuts.close') }}
      </FdButton>
    </template>
  </FdModal>
</template>

<style scoped>
.fd-shortcuts {
  display: flex;
  flex-direction: column;
}

.fd-shortcuts__row {
  display: flex;
  align-items: center;
  gap: var(--fd-space-3);
  min-height: 38px;
  border-bottom: 1px solid var(--fd-border-subtle);
}

.fd-shortcuts__row:last-child {
  border-bottom: 0;
}

.fd-shortcuts__keys {
  display: inline-flex;
  flex: 0 0 132px;
}

.fd-shortcuts__label {
  flex: 1;
  font-size: var(--fd-text-13);
}

.fd-shortcuts__status {
  display: inline-flex;
  flex: none;
  align-items: center;
  gap: 4px;
  color: var(--fd-text-muted);
  font-size: var(--fd-text-11);
}

.fd-shortcuts__status--ready {
  color: var(--fd-success);
}
</style>
