<script setup lang="ts">
import { onMounted, ref } from 'vue'

const props = withDefaults(
  defineProps<{
    placeholder?: string
    rows?: number
    invalid?: boolean
    disabled?: boolean
    autofocus?: boolean
    ariaLabel?: string
  }>(),
  {
    placeholder: '',
    rows: 4,
    invalid: false,
    disabled: false,
    autofocus: false,
    ariaLabel: undefined,
  },
)

const model = defineModel<string>({ default: '' })
const textarea = ref<HTMLTextAreaElement | null>(null)

onMounted(() => {
  if (props.autofocus) {
    textarea.value?.focus()
  }
})
</script>

<template>
  <textarea
    ref="textarea"
    v-model="model"
    :rows="rows"
    :placeholder="placeholder"
    :disabled="disabled"
    :aria-label="ariaLabel"
    :aria-invalid="invalid || undefined"
    class="fd-textarea"
    :class="{ 'fd-textarea--invalid': invalid }"
  />
</template>

<style scoped>
.fd-textarea {
  width: 100%;
  min-height: 64px;
  padding: 8px 10px;
  border: 1px solid var(--fd-border);
  border-radius: var(--fd-radius-control);
  background: var(--fd-surface);
  color: var(--fd-text);
  font-family: var(--fd-font-sans);
  font-size: var(--fd-text-13);
  line-height: var(--fd-line-body);
  resize: vertical;
  transition:
    border-color var(--fd-motion-hover) var(--fd-ease-standard),
    box-shadow var(--fd-motion-hover) var(--fd-ease-standard);
}

.fd-textarea::placeholder {
  color: var(--fd-text-muted);
}

.fd-textarea:hover:not(:disabled) {
  border-color: var(--fd-text-muted);
}

.fd-textarea:focus {
  outline: none;
  border-color: var(--fd-accent);
  box-shadow: 0 0 0 3px var(--fd-focus-ring);
}

.fd-textarea--invalid,
.fd-textarea--invalid:hover:not(:disabled) {
  border-color: var(--fd-danger);
}

.fd-textarea:disabled {
  cursor: not-allowed;
  opacity: 0.6;
}
</style>
