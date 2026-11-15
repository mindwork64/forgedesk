<script setup lang="ts">
withDefaults(
  defineProps<{
    tone?: 'neutral' | 'accent' | 'success' | 'warning' | 'danger' | 'info'
  }>(),
  { tone: 'neutral' },
)
</script>

<template>
  <span class="fd-badge" :class="`fd-badge--${tone}`">
    <slot />
  </span>
</template>

<style scoped>
.fd-badge {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  height: 20px;
  padding: 0 7px;
  border: 1px solid var(--fd-border);
  border-radius: 999px;
  background: var(--fd-surface);
  color: var(--fd-text-secondary);
  font-size: var(--fd-text-11);
  font-weight: var(--fd-weight-medium);
  white-space: nowrap;
}

.fd-badge--accent {
  border-color: transparent;
  background: var(--fd-accent-soft);
  color: var(--fd-accent);
}

.fd-badge--success {
  border-color: transparent;
  background: var(--fd-success-soft);
  color: var(--fd-success);
}

.fd-badge--warning {
  border-color: transparent;
  background: var(--fd-warning-soft);
  color: var(--fd-warning);
}

.fd-badge--danger {
  border-color: transparent;
  background: var(--fd-danger-soft);
  color: var(--fd-danger);
}

.fd-badge--info {
  border-color: transparent;
  background: var(--fd-info-soft);
  color: var(--fd-info);
}
</style>
