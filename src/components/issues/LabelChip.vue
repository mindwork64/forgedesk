<script setup lang="ts">
import { statusColorVar } from '@/lib/issue'
import type { Label } from '@/types/ipc'

defineProps<{ label: Label }>()
</script>

<template>
  <span class="fd-label-chip">
    <span
      class="fd-label-chip__dot"
      :style="{ background: statusColorVar(label.color) }"
      aria-hidden="true"
    />
    {{ label.name }}
  </span>
</template>

<style scoped>
.fd-label-chip {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  height: 20px;
  max-width: 140px;
  padding: 0 7px;
  overflow: hidden;
  border: 1px solid var(--fd-border);
  border-radius: 999px;
  background: var(--fd-surface);
  color: var(--fd-text-secondary);
  font-size: var(--fd-text-11);
  text-overflow: ellipsis;
  white-space: nowrap;
}

.fd-label-chip__dot {
  flex: none;
  width: 5px;
  height: 5px;
  border-radius: 50%;
}
</style>
