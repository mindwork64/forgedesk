<script setup lang="ts">
import { LoaderCircle } from '@lucide/vue'

withDefaults(
  defineProps<{
    variant?: 'primary' | 'secondary' | 'ghost' | 'destructive'
    size?: 'md' | 'sm'
    type?: 'button' | 'submit' | 'reset'
    disabled?: boolean
    loading?: boolean
  }>(),
  {
    variant: 'secondary',
    size: 'md',
    type: 'button',
    disabled: false,
    loading: false,
  },
)
</script>

<template>
  <button
    :type="type"
    :disabled="disabled || loading"
    :aria-busy="loading || undefined"
    class="fd-button"
    :class="[`fd-button--${variant}`, `fd-button--${size}`]"
  >
    <LoaderCircle v-if="loading" :size="14" class="fd-button__spinner" aria-hidden="true" />
    <slot />
  </button>
</template>

<style scoped>
.fd-button {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  height: var(--fd-button-height);
  padding: 0 12px;
  border: 1px solid transparent;
  border-radius: var(--fd-radius-control);
  font-size: var(--fd-text-13);
  font-weight: var(--fd-weight-medium);
  line-height: 1;
  white-space: nowrap;
  user-select: none;
  transition:
    background-color var(--fd-motion-hover) var(--fd-ease-standard),
    border-color var(--fd-motion-hover) var(--fd-ease-standard),
    color var(--fd-motion-hover) var(--fd-ease-standard);
}

.fd-button--sm {
  height: var(--fd-button-height-sm);
  padding: 0 10px;
  font-size: var(--fd-text-12);
}

.fd-button--primary {
  background: var(--fd-accent);
  color: var(--fd-accent-contrast);
}

.fd-button--primary:hover:not(:disabled) {
  background: var(--fd-accent-hover);
}

.fd-button--secondary {
  background: var(--fd-surface);
  border-color: var(--fd-border);
  color: var(--fd-text);
}

.fd-button--secondary:hover:not(:disabled) {
  background: var(--fd-surface-hover);
  border-color: var(--fd-border);
}

.fd-button--ghost {
  background: transparent;
  color: var(--fd-text-secondary);
}

.fd-button--ghost:hover:not(:disabled) {
  background: var(--fd-surface-hover);
  color: var(--fd-text);
}

.fd-button--destructive {
  background: var(--fd-danger);
  color: var(--fd-danger-contrast);
}

.fd-button--destructive:hover:not(:disabled) {
  background: var(--fd-danger-hover);
}

.fd-button:disabled {
  cursor: not-allowed;
  opacity: 0.5;
}

.fd-button:focus-visible {
  outline: 2px solid var(--fd-accent);
  outline-offset: 1px;
}

.fd-button__spinner {
  animation: fd-spin 0.9s linear infinite;
}

@keyframes fd-spin {
  to {
    transform: rotate(360deg);
  }
}
</style>
