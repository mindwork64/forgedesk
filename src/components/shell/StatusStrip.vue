<script setup lang="ts">
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'

import FdPopover from '../ui/FdPopover.vue'
import LocalStatusDetails from './LocalStatusDetails.vue'

const { t } = useI18n()
const detailsOpen = ref(false)
</script>

<template>
  <footer class="fd-status-strip" data-testid="status-strip">
    <FdPopover
      v-model:open="detailsOpen"
      :width="330"
      align="start"
      placement="top"
      :aria-label="t('status.details')"
    >
      <template #trigger="{ toggle }">
        <button
          type="button"
          class="fd-status-strip__state"
          :aria-expanded="detailsOpen"
          aria-haspopup="dialog"
          aria-live="polite"
          @click="toggle"
        >
          <span class="fd-status-strip__dot" aria-hidden="true" />
          <span>{{ t('status.localReady') }}</span>
        </button>
      </template>
      <LocalStatusDetails />
    </FdPopover>

    <div class="fd-status-strip__right">
      <span>{{ t('status.localOnly') }}</span>
      <span class="fd-status-strip__separator" aria-hidden="true">·</span>
      <span>{{ t('status.noIntegrations') }}</span>
    </div>
  </footer>
</template>

<style scoped>
.fd-status-strip {
  display: flex;
  flex: 0 0 var(--fd-status-height);
  align-items: center;
  justify-content: space-between;
  height: var(--fd-status-height);
  padding: 0 var(--fd-space-3);
  border-top: 1px solid var(--fd-border);
  background: var(--fd-sidebar);
  color: var(--fd-text-muted);
  font-size: var(--fd-text-11);
}

.fd-status-strip__state {
  display: flex;
  align-items: center;
  gap: 5px;
  height: 20px;
  padding: 0 4px;
  border: 0;
  border-radius: 4px;
  background: transparent;
  color: var(--fd-text-secondary);
  font-size: var(--fd-text-11);
}

.fd-status-strip__state:hover {
  background: var(--fd-surface-hover);
  color: var(--fd-text);
}

.fd-status-strip__dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--fd-success);
}

.fd-status-strip__right {
  display: flex;
  align-items: center;
  gap: 5px;
  white-space: nowrap;
}

.fd-status-strip__separator {
  color: var(--fd-border);
}
</style>
