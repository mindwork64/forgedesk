<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'

import { planningStatusLabel } from '@/lib/planning'
import type { PlanningStatus } from '@/types/ipc'

const props = defineProps<{ status: PlanningStatus }>()
const { t } = useI18n()
const label = computed(() => planningStatusLabel(props.status, t))
</script>

<template>
  <span class="fd-planning-status" :class="`fd-planning-status--${status}`">
    <span class="fd-planning-status__dot" aria-hidden="true" />
    {{ label }}
  </span>
</template>

<style scoped>
.fd-planning-status {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  color: var(--fd-text-secondary);
  font-size: var(--fd-text-11);
  white-space: nowrap;
}

.fd-planning-status__dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--fd-info);
}

.fd-planning-status--active .fd-planning-status__dot {
  background: var(--fd-accent);
}

.fd-planning-status--paused .fd-planning-status__dot {
  background: var(--fd-warning);
}

.fd-planning-status--completed .fd-planning-status__dot {
  background: var(--fd-success);
}

.fd-planning-status--canceled {
  color: var(--fd-text-muted);
}

.fd-planning-status--canceled .fd-planning-status__dot {
  background: var(--fd-text-muted);
}
</style>
