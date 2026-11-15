<script setup lang="ts">
import type { Component } from 'vue'

withDefaults(
  defineProps<{
    icon: Component
    title: string
    body?: string
  }>(),
  { body: undefined },
)
</script>

<template>
  <div class="fd-empty">
    <component :is="icon" :size="28" class="fd-empty__icon" aria-hidden="true" />
    <h3 class="fd-empty__title">{{ title }}</h3>
    <p v-if="body" class="fd-empty__body">{{ body }}</p>
    <div v-if="$slots.default" class="fd-empty__actions">
      <slot />
    </div>
  </div>
</template>

<style scoped>
.fd-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: var(--fd-space-2);
  padding: var(--fd-space-9) var(--fd-space-6);
  text-align: center;
}

.fd-empty__icon {
  margin-bottom: var(--fd-space-1);
  color: var(--fd-text-muted);
}

.fd-empty__title {
  font-size: var(--fd-text-14);
  font-weight: var(--fd-weight-semibold);
  line-height: var(--fd-line-tight);
}

.fd-empty__body {
  max-width: 440px;
  color: var(--fd-text-secondary);
  font-size: var(--fd-text-13);
  line-height: var(--fd-line-body);
}

.fd-empty__actions {
  display: flex;
  align-items: center;
  gap: var(--fd-space-2);
  margin-top: var(--fd-space-2);
}
</style>
