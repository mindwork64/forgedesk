<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'

import { statusColorVar, statusLabel } from '@/lib/issue'
import type { StatusSummary } from '@/types/ipc'

const props = defineProps<{ status: StatusSummary }>()
const { t, te } = useI18n()

const label = computed(() => statusLabel(props.status, t, te))
</script>

<template>
  <span class="fd-status-badge">
    <span
      class="fd-status-badge__dot"
      :style="{ background: statusColorVar(status.color) }"
      aria-hidden="true"
    />
    <span class="fd-status-badge__label">{{ label }}</span>
  </span>
</template>

<style scoped>
.fd-status-badge {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  min-width: 0;
  color: var(--fd-text-secondary);
  font-size: var(--fd-text-12);
}

.fd-status-badge__dot {
  flex: none;
  width: 6px;
  height: 6px;
  border-radius: 50%;
}

.fd-status-badge__label {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
</style>
