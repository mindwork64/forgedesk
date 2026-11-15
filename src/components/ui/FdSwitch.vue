<script setup lang="ts">
const props = withDefaults(
  defineProps<{
    label: string
    disabled?: boolean
  }>(),
  { disabled: false },
)

const model = defineModel<boolean>({ required: true })

function toggle(): void {
  if (!props.disabled) {
    model.value = !model.value
  }
}
</script>

<template>
  <button
    type="button"
    role="switch"
    :aria-checked="model"
    :aria-label="label"
    :disabled="disabled"
    class="fd-switch"
    :class="{ 'fd-switch--checked': model }"
    @click="toggle"
  >
    <span class="fd-switch__thumb" aria-hidden="true" />
  </button>
</template>

<style scoped>
.fd-switch {
  position: relative;
  display: inline-flex;
  flex: none;
  width: 36px;
  height: 20px;
  padding: 2px;
  border: 1px solid var(--fd-border);
  border-radius: 999px;
  background: var(--fd-surface-selected);
  transition:
    background-color var(--fd-motion-hover) var(--fd-ease-standard),
    border-color var(--fd-motion-hover) var(--fd-ease-standard);
}

.fd-switch--checked {
  border-color: var(--fd-accent);
  background: var(--fd-accent);
}

.fd-switch__thumb {
  width: 14px;
  height: 14px;
  border-radius: 50%;
  background: var(--fd-text);
  box-shadow: var(--fd-shadow-control);
  transform: translateX(0);
  transition: transform var(--fd-motion-hover) var(--fd-ease-standard);
}

.fd-switch--checked .fd-switch__thumb {
  background: var(--fd-accent-contrast);
  transform: translateX(16px);
}

.fd-switch:disabled {
  cursor: not-allowed;
  opacity: 0.5;
}
</style>
