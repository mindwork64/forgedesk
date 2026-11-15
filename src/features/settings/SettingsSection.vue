<script setup lang="ts">
defineProps<{
  title: string
  description?: string
}>()
</script>

<template>
  <section class="fd-settings-section">
    <header class="fd-settings-section__header">
      <div class="fd-settings-section__text">
        <h2 class="fd-settings-section__title">{{ title }}</h2>
        <p v-if="description" class="fd-settings-section__description">
          {{ description }}
        </p>
      </div>
      <div v-if="$slots.actions" class="fd-settings-section__actions">
        <slot name="actions" />
      </div>
    </header>
    <div class="fd-settings-section__body">
      <slot />
    </div>
  </section>
</template>

<style scoped>
.fd-settings-section {
  display: flex;
  flex-direction: column;
  gap: var(--fd-space-2);
}

.fd-settings-section__header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: var(--fd-space-3);
  padding-bottom: var(--fd-space-2);
}

.fd-settings-section__text {
  min-width: 0;
}

.fd-settings-section__title {
  font-size: var(--fd-text-14);
  font-weight: var(--fd-weight-semibold);
  line-height: var(--fd-line-tight);
}

.fd-settings-section__description {
  margin-top: 2px;
  color: var(--fd-text-secondary);
  font-size: var(--fd-text-12);
  line-height: var(--fd-line-body);
}

.fd-settings-section__actions {
  display: flex;
  flex: none;
  align-items: center;
  gap: var(--fd-space-2);
}

.fd-settings-section__body {
  display: flex;
  flex-direction: column;
}
</style>
