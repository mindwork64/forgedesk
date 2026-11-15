<script setup lang="ts">
import { ChevronDown, ChevronsUp, ChevronUp, Minus } from '@lucide/vue'
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'

import { priorityLabel } from '@/lib/issue'
import type { Priority } from '@/types/ipc'

const props = withDefaults(
  defineProps<{ priority: Priority; showLabel?: boolean }>(),
  { showLabel: false },
)
const { t } = useI18n()

const icon = computed(() => {
  switch (props.priority) {
    case 'urgent':
      return ChevronsUp
    case 'high':
      return ChevronUp
    case 'medium':
      return Minus
    case 'low':
      return ChevronDown
    default:
      return null
  }
})

const label = computed(() => priorityLabel(props.priority, t))
</script>

<template>
  <span
    class="fd-priority"
    :class="`fd-priority--${priority}`"
    :title="label"
    :aria-label="label"
  >
    <component :is="icon" v-if="icon" :size="14" aria-hidden="true" />
    <span v-if="showLabel" class="fd-priority__label">{{ label }}</span>
    <span v-else-if="!icon" class="fd-priority__none" aria-hidden="true">—</span>
  </span>
</template>

<style scoped>
.fd-priority {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  color: var(--fd-text-muted);
}

.fd-priority--urgent {
  color: var(--fd-danger);
}

.fd-priority--high {
  color: var(--fd-warning);
}

.fd-priority--medium {
  color: var(--fd-info);
}

.fd-priority--low {
  color: var(--fd-text-muted);
}

.fd-priority__label {
  color: var(--fd-text-secondary);
  font-size: var(--fd-text-12);
}

.fd-priority__none {
  color: var(--fd-text-muted);
  font-size: var(--fd-text-12);
}
</style>
