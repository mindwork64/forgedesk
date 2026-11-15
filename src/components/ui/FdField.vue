<script setup lang="ts">
import { computed, useId } from 'vue'

const props = defineProps<{
  label: string
  hint?: string
  error?: string
  required?: boolean
  id?: string
}>()

const generatedId = useId()
const fieldId = computed(() => props.id ?? `fd-field-${generatedId}`)
const hintId = computed(() => `${fieldId.value}-hint`)
const errorId = computed(() => `${fieldId.value}-error`)
const describedBy = computed(() => {
  if (props.error) {
    return errorId.value
  }
  return props.hint ? hintId.value : undefined
})
</script>

<template>
  <div class="fd-field">
    <label class="fd-field__label" :for="fieldId">
      {{ label }}
      <span v-if="required" class="fd-field__required" aria-hidden="true">*</span>
    </label>
    <slot :id="fieldId" :describedBy="describedBy" />
    <p v-if="error" :id="errorId" class="fd-field__error" role="alert">
      {{ error }}
    </p>
    <p v-else-if="hint" :id="hintId" class="fd-field__hint">{{ hint }}</p>
  </div>
</template>

<style scoped>
.fd-field {
  display: flex;
  flex-direction: column;
  gap: 6px;
  min-width: 0;
}

.fd-field__label {
  color: var(--fd-text-secondary);
  font-size: var(--fd-text-12);
  font-weight: var(--fd-weight-medium);
  line-height: var(--fd-line-tight);
}

.fd-field__required {
  margin-left: 2px;
  color: var(--fd-danger);
}

.fd-field__hint,
.fd-field__error {
  font-size: var(--fd-text-12);
  line-height: var(--fd-line-body);
}

.fd-field__hint {
  color: var(--fd-text-muted);
}

.fd-field__error {
  color: var(--fd-danger);
}
</style>
