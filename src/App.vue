<script setup lang="ts">
import { Layers } from '@lucide/vue'
import { onMounted } from 'vue'
import { useI18n } from 'vue-i18n'

import OnboardingFlow from '@/components/OnboardingFlow.vue'
import AppShell from '@/components/shell/AppShell.vue'
import StartupError from '@/components/StartupError.vue'
import FdLoadingSkeleton from '@/components/ui/FdLoadingSkeleton.vue'
import FdToastHost from '@/components/ui/FdToastHost.vue'
import { useAppStore } from '@/stores/app'

const app = useAppStore()
const { t } = useI18n()

onMounted(() => {
  void app.load()
})
</script>

<template>
  <div v-if="app.loading" class="fd-boot" data-testid="boot">
    <div class="fd-boot__brand">
      <span class="fd-boot__mark">
        <Layers :size="18" aria-hidden="true" />
      </span>
      <span>{{ t('app.wordmark') }}</span>
    </div>
    <p class="fd-boot__label">{{ t('app.loading') }}</p>
    <div class="fd-boot__skeleton">
      <FdLoadingSkeleton :rows="3" :height="32" />
    </div>
  </div>

  <StartupError
    v-else-if="app.startupError"
    :error="app.startupError"
    @retry="app.load()"
  />

  <template v-else-if="app.data">
    <AppShell />
    <OnboardingFlow v-if="app.needsOnboarding" />
    <FdToastHost />
  </template>

  <StartupError
    v-else
    :error="{
      code: 'internal',
      message: 'The application state is unavailable.',
    }"
    @retry="app.load()"
  />
</template>

<style scoped>
.fd-boot {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: var(--fd-space-3);
  height: 100vh;
  background: var(--fd-bg);
}

.fd-boot__brand {
  display: flex;
  align-items: center;
  gap: 8px;
  color: var(--fd-text);
  font-size: var(--fd-text-14);
  font-weight: var(--fd-weight-semibold);
}

.fd-boot__mark {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border-radius: var(--fd-radius-control);
  background: var(--fd-accent-soft);
  color: var(--fd-accent);
}

.fd-boot__label {
  color: var(--fd-text-muted);
  font-size: var(--fd-text-12);
}

.fd-boot__skeleton {
  width: 320px;
  margin-top: var(--fd-space-2);
}
</style>
