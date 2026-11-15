<script setup lang="ts">
import { ChevronDown } from '@lucide/vue'
import { useAttrs } from 'vue'

defineOptions({ inheritAttrs: false })

withDefaults(
  defineProps<{
    disabled?: boolean
    invalid?: boolean
    ariaLabel?: string
  }>(),
  {
    disabled: false,
    invalid: false,
    ariaLabel: undefined,
  },
)

const model = defineModel<string>({ required: true })
const attrs = useAttrs()
</script>

<template>
  <span class="fd-select">
    <select
      v-bind="attrs"
      v-model="model"
      :disabled="disabled"
      :aria-label="ariaLabel"
      :aria-invalid="invalid || undefined"
      class="fd-select__control"
      :class="{ 'fd-select__control--invalid': invalid }"
    >
      <slot />
    </select>
    <ChevronDown :size="14" class="fd-select__chevron" aria-hidden="true" />
  </span>
</template>

<style scoped>
.fd-select {
  position: relative;
  display: inline-flex;
  width: 100%;
}

.fd-select__control {
  width: 100%;
  height: var(--fd-input-height);
  padding: 0 30px 0 10px;
  border: 1px solid var(--fd-border);
  border-radius: var(--fd-radius-control);
  background: var(--fd-surface);
  color: var(--fd-text);
  font-size: var(--fd-text-13);
  appearance: none;
  cursor: pointer;
  transition:
    border-color var(--fd-motion-hover) var(--fd-ease-standard),
    box-shadow var(--fd-motion-hover) var(--fd-ease-standard);
}

.fd-select__control:hover:not(:disabled) {
  border-color: var(--fd-text-muted);
}

.fd-select__control:focus {
  outline: none;
  border-color: var(--fd-accent);
  box-shadow: 0 0 0 3px var(--fd-focus-ring);
}

.fd-select__control--invalid {
  border-color: var(--fd-danger);
}

.fd-select__control:disabled {
  cursor: not-allowed;
  opacity: 0.6;
}

.fd-select__chevron {
  position: absolute;
  top: 50%;
  right: 9px;
  transform: translateY(-50%);
  color: var(--fd-text-muted);
  pointer-events: none;
}
</style>
