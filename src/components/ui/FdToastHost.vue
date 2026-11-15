<script setup lang="ts">
import {
  CircleAlert,
  CircleCheck,
  Info,
  TriangleAlert,
  X,
} from '@lucide/vue'
import { useI18n } from 'vue-i18n'

import { useToastStore } from '@/stores/toasts'

const toast = useToastStore()
const { t } = useI18n()

const icons = {
  success: CircleCheck,
  error: CircleAlert,
  warning: TriangleAlert,
  info: Info,
} as const
</script>

<template>
  <Teleport to="body">
    <div class="fd-toast-host" aria-live="polite">
      <TransitionGroup name="fd-toast" tag="div" class="fd-toast-list">
        <div
          v-for="item in toast.items"
          :key="item.id"
          class="fd-toast"
          :class="`fd-toast--${item.kind}`"
          :role="item.kind === 'error' ? 'alert' : 'status'"
        >
          <component
            :is="icons[item.kind]"
            :size="16"
            class="fd-toast__icon"
            aria-hidden="true"
          />
          <div class="fd-toast__content">
            <p class="fd-toast__title">{{ item.title }}</p>
            <p v-if="item.message" class="fd-toast__message">
              {{ item.message }}
            </p>
            <details v-if="item.detail" class="fd-toast__details">
              <summary>{{ t('errors.details') }}</summary>
              <pre>{{ item.detail }}</pre>
            </details>
          </div>
          <button
            type="button"
            class="fd-toast__close"
            :aria-label="t('common.dismiss')"
            @click="toast.dismiss(item.id)"
          >
            <X :size="14" />
          </button>
        </div>
      </TransitionGroup>
    </div>
  </Teleport>
</template>

<style scoped>
.fd-toast-host {
  position: fixed;
  right: var(--fd-space-4);
  bottom: calc(var(--fd-status-height) + var(--fd-space-3));
  z-index: var(--fd-z-toast);
  display: flex;
  justify-content: flex-end;
  pointer-events: none;
}

.fd-toast-list {
  display: flex;
  flex-direction: column;
  gap: var(--fd-space-2);
  width: min(360px, calc(100vw - 32px));
}

.fd-toast {
  display: flex;
  align-items: flex-start;
  gap: var(--fd-space-2);
  padding: 10px 10px 10px 12px;
  border: 1px solid var(--fd-border);
  border-radius: var(--fd-radius-panel);
  background: var(--fd-surface-raised);
  box-shadow: var(--fd-shadow-popover);
  pointer-events: auto;
}

.fd-toast__icon {
  flex: none;
  margin-top: 1px;
}

.fd-toast--success .fd-toast__icon {
  color: var(--fd-success);
}

.fd-toast--error .fd-toast__icon {
  color: var(--fd-danger);
}

.fd-toast--warning .fd-toast__icon {
  color: var(--fd-warning);
}

.fd-toast--info .fd-toast__icon {
  color: var(--fd-info);
}

.fd-toast__content {
  flex: 1 1 auto;
  min-width: 0;
}

.fd-toast__title {
  font-size: var(--fd-text-13);
  font-weight: var(--fd-weight-medium);
  line-height: var(--fd-line-tight);
}

.fd-toast__message {
  margin-top: 2px;
  color: var(--fd-text-secondary);
  font-size: var(--fd-text-12);
  line-height: var(--fd-line-body);
  overflow-wrap: anywhere;
}

.fd-toast__details {
  margin-top: 4px;
  color: var(--fd-text-muted);
  font-size: var(--fd-text-11);
}

.fd-toast__details summary {
  cursor: pointer;
}

.fd-toast__details pre {
  margin: 4px 0 0;
  padding: 6px;
  border-radius: 4px;
  background: var(--fd-bg);
  font-family: var(--fd-font-mono);
  font-size: 11px;
  white-space: pre-wrap;
  word-break: break-word;
}

.fd-toast__close {
  display: inline-flex;
  flex: none;
  align-items: center;
  justify-content: center;
  width: 22px;
  height: 22px;
  padding: 0;
  border: 0;
  border-radius: 4px;
  background: transparent;
  color: var(--fd-text-muted);
}

.fd-toast__close:hover {
  background: var(--fd-surface-hover);
  color: var(--fd-text);
}

.fd-toast-enter-active,
.fd-toast-leave-active {
  transition:
    opacity var(--fd-motion-drawer) var(--fd-ease-standard),
    transform var(--fd-motion-drawer) var(--fd-ease-standard);
}

.fd-toast-enter-from,
.fd-toast-leave-to {
  opacity: 0;
  transform: translateX(8px);
}
</style>
