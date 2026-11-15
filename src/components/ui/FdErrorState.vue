<script setup lang="ts">
import { CircleAlert } from '@lucide/vue'
import { useI18n } from 'vue-i18n'

import FdButton from './FdButton.vue'

withDefaults(
  defineProps<{
    title: string
    message: string
    code?: string
    detail?: string
    retryLabel?: string
    showRetry?: boolean
  }>(),
  {
    code: undefined,
    detail: undefined,
    retryLabel: undefined,
    showRetry: true,
  },
)

const emit = defineEmits<{ (event: 'retry'): void }>()
const { t } = useI18n()
</script>

<template>
  <div class="fd-error" role="alert">
    <CircleAlert :size="28" class="fd-error__icon" aria-hidden="true" />
    <h2 class="fd-error__title">{{ title }}</h2>
    <p class="fd-error__message">{{ message }}</p>
    <div class="fd-error__actions">
      <FdButton v-if="showRetry" variant="secondary" @click="emit('retry')">
        {{ retryLabel ?? t('common.retry') }}
      </FdButton>
      <slot />
    </div>
    <details v-if="detail || code" class="fd-error__details">
      <summary>{{ t('errors.details') }}</summary>
      <p v-if="code" class="fd-error__code">
        {{ t('errors.code') }}: <span class="fd-mono">{{ code }}</span>
      </p>
      <pre v-if="detail" class="fd-error__detail">{{ detail }}</pre>
    </details>
  </div>
</template>

<style scoped>
.fd-error {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: var(--fd-space-2);
  max-width: 520px;
}

.fd-error__icon {
  color: var(--fd-danger);
}

.fd-error__title {
  font-size: var(--fd-text-16);
  font-weight: var(--fd-weight-semibold);
  line-height: var(--fd-line-tight);
}

.fd-error__message {
  color: var(--fd-text-secondary);
  font-size: var(--fd-text-13);
  line-height: var(--fd-line-body);
}

.fd-error__actions {
  display: flex;
  gap: var(--fd-space-2);
  margin-top: var(--fd-space-2);
}

.fd-error__details {
  width: 100%;
  margin-top: var(--fd-space-3);
  color: var(--fd-text-muted);
  font-size: var(--fd-text-12);
}

.fd-error__details summary {
  cursor: pointer;
  user-select: none;
}

.fd-error__code {
  margin-top: var(--fd-space-2);
}

.fd-error__detail {
  margin: var(--fd-space-2) 0 0;
  padding: var(--fd-space-2);
  border: 1px solid var(--fd-border);
  border-radius: var(--fd-radius-control);
  background: var(--fd-surface);
  color: var(--fd-text-secondary);
  font-family: var(--fd-font-mono);
  font-size: var(--fd-text-11);
  white-space: pre-wrap;
  word-break: break-word;
}
</style>
