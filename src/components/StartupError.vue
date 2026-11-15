<script setup lang="ts">
import { Layers } from '@lucide/vue'
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'

import FdErrorState from '@/components/ui/FdErrorState.vue'
import type { IpcError } from '@/types/ipc'

const props = defineProps<{ error: IpcError }>()
const emit = defineEmits<{ (event: 'retry'): void }>()
const { t } = useI18n()

const desktopUnavailable = computed(
  () => props.error.code === 'desktop_unavailable',
)

const title = computed(() =>
  desktopUnavailable.value
    ? t('errors.desktopTitle')
    : t('errors.startupTitle'),
)

const message = computed(() =>
  desktopUnavailable.value
    ? t('errors.desktopBody')
    : t('errors.startupBody'),
)

function retry(): void {
  if (desktopUnavailable.value) {
    window.location.reload()
    return
  }
  emit('retry')
}
</script>

<template>
  <div class="fd-startup-error" data-testid="startup-error">
    <div class="fd-startup-error__brand">
      <span class="fd-startup-error__mark">
        <Layers :size="18" aria-hidden="true" />
      </span>
      <span>{{ t('app.wordmark') }}</span>
    </div>
    <FdErrorState
      :title="title"
      :message="message"
      :code="error.code"
      :detail="error.detail"
      @retry="retry"
    />
  </div>
</template>

<style scoped>
.fd-startup-error {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  justify-content: center;
  gap: var(--fd-space-6);
  height: 100vh;
  padding: var(--fd-space-9);
  background: var(--fd-bg);
}

.fd-startup-error__brand {
  display: flex;
  align-items: center;
  gap: 8px;
  color: var(--fd-text);
  font-size: var(--fd-text-14);
  font-weight: var(--fd-weight-semibold);
}

.fd-startup-error__mark {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border-radius: var(--fd-radius-control);
  background: var(--fd-accent-soft);
  color: var(--fd-accent);
}
</style>
