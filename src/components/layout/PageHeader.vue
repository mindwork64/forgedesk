<script setup lang="ts">
defineProps<{
  title: string
  description?: string
}>()
</script>

<template>
  <header class="fd-page-header" data-testid="page-header">
    <div class="fd-page-header__text">
      <h1 class="fd-page-header__title">{{ title }}</h1>
      <p v-if="description" class="fd-page-header__description">
        {{ description }}
      </p>
    </div>
    <div v-if="$slots.actions" class="fd-page-header__actions">
      <slot name="actions" />
    </div>
  </header>
</template>

<style scoped>
.fd-page-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: var(--fd-space-4);
  padding: 16px var(--fd-content-pad) 12px;
}

.fd-page-header__text {
  min-width: 0;
}

.fd-page-header__title {
  font-size: var(--fd-text-20);
  font-weight: var(--fd-weight-semibold);
  line-height: var(--fd-line-tight);
  letter-spacing: -0.01em;
}

.fd-page-header__description {
  max-width: 720px;
  margin-top: 4px;
  color: var(--fd-text-secondary);
  font-size: var(--fd-text-13);
  line-height: var(--fd-line-body);
}

.fd-page-header__actions {
  display: flex;
  flex: none;
  align-items: center;
  gap: var(--fd-space-2);
  padding-top: 2px;
}
</style>
