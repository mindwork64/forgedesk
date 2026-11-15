<script setup lang="ts">
import { onMounted, ref } from 'vue'

const props = withDefaults(
  defineProps<{
    placeholder?: string
    type?: 'text' | 'search' | 'date' | 'number' | 'email'
    invalid?: boolean
    disabled?: boolean
    autofocus?: boolean
    ariaLabel?: string
  }>(),
  {
    placeholder: '',
    type: 'text',
    invalid: false,
    disabled: false,
    autofocus: false,
    ariaLabel: undefined,
  },
)

const model = defineModel<string>({ default: '' })
const input = ref<HTMLInputElement | null>(null)

onMounted(() => {
  if (props.autofocus) {
    input.value?.focus()
  }
})

defineExpose({ focus: () => input.value?.focus() })
</script>

<template>
  <input
    ref="input"
    v-model="model"
    :type="type"
    :placeholder="placeholder"
    :disabled="disabled"
    :aria-label="ariaLabel"
    :aria-invalid="invalid || undefined"
    class="fd-input"
    :class="{ 'fd-input--invalid': invalid }"
  />
</template>

<style scoped>
.fd-input {
  width: 100%;
  height: var(--fd-input-height);
  padding: 0 10px;
  border: 1px solid var(--fd-border);
  border-radius: var(--fd-radius-control);
  background: var(--fd-surface);
  color: var(--fd-text);
  font-size: var(--fd-text-13);
  transition:
    border-color var(--fd-motion-hover) var(--fd-ease-standard),
    box-shadow var(--fd-motion-hover) var(--fd-ease-standard);
}

.fd-input::placeholder {
  color: var(--fd-text-muted);
}

.fd-input:hover:not(:disabled) {
  border-color: var(--fd-text-muted);
}

.fd-input:focus {
  outline: none;
  border-color: var(--fd-accent);
  box-shadow: 0 0 0 3px var(--fd-focus-ring);
}

.fd-input--invalid,
.fd-input--invalid:hover:not(:disabled) {
  border-color: var(--fd-danger);
}

.fd-input--invalid:focus {
  border-color: var(--fd-danger);
  box-shadow: 0 0 0 3px var(--fd-danger-soft);
}

.fd-input:disabled {
  cursor: not-allowed;
  opacity: 0.6;
}
</style>
