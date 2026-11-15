<script setup lang="ts">
withDefaults(
  defineProps<{
    label: string
    tooltip?: string | false
    size?: 'md' | 'sm'
    variant?: 'ghost' | 'secondary'
    disabled?: boolean
    pressed?: boolean
    type?: 'button' | 'submit'
  }>(),
  {
    tooltip: undefined,
    size: 'sm',
    variant: 'ghost',
    disabled: false,
    pressed: undefined,
    type: 'button',
  },
)
</script>

<template>
  <button
    v-tooltip="tooltip === false ? false : (tooltip ?? label)"
    :type="type"
    :aria-label="label"
    :aria-pressed="pressed"
    :disabled="disabled"
    class="fd-icon-button"
    :class="[`fd-icon-button--${size}`, `fd-icon-button--${variant}`]"
  >
    <slot />
  </button>
</template>

<style scoped>
.fd-icon-button {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex: none;
  width: 28px;
  height: 28px;
  padding: 0;
  border: 1px solid transparent;
  border-radius: var(--fd-radius-control);
  background: transparent;
  color: var(--fd-text-secondary);
  transition:
    background-color var(--fd-motion-hover) var(--fd-ease-standard),
    color var(--fd-motion-hover) var(--fd-ease-standard);
}

.fd-icon-button--md {
  width: 32px;
  height: 32px;
}

.fd-icon-button--secondary {
  border-color: var(--fd-border);
  background: var(--fd-surface);
  color: var(--fd-text);
}

.fd-icon-button:hover:not(:disabled) {
  background: var(--fd-surface-hover);
  color: var(--fd-text);
}

.fd-icon-button[aria-pressed='true'] {
  background: var(--fd-surface-selected);
  color: var(--fd-accent);
}

.fd-icon-button:disabled {
  cursor: not-allowed;
  opacity: 0.5;
}
</style>
