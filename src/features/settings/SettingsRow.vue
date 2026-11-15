<script setup lang="ts">
withDefaults(
  defineProps<{
    label: string
    hint?: string
    stacked?: boolean
  }>(),
  { hint: undefined, stacked: false },
)
</script>

<template>
  <div class="fd-settings-row" :class="{ 'fd-settings-row--stacked': stacked }">
    <div class="fd-settings-row__text">
      <span class="fd-settings-row__label">{{ label }}</span>
      <p v-if="hint" class="fd-settings-row__hint">{{ hint }}</p>
    </div>
    <div class="fd-settings-row__control">
      <slot />
    </div>
  </div>
</template>

<style scoped>
.fd-settings-row {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: var(--fd-space-6);
  padding: 12px 0;
  border-bottom: 1px solid var(--fd-border-subtle);
}

.fd-settings-row:last-child {
  border-bottom: 0;
  padding-bottom: 0;
}

.fd-settings-row--stacked {
  flex-direction: column;
  gap: var(--fd-space-2);
}

.fd-settings-row__text {
  min-width: 0;
  padding-top: 5px;
}

.fd-settings-row--stacked .fd-settings-row__text {
  padding-top: 0;
}

.fd-settings-row__label {
  font-size: var(--fd-text-13);
  font-weight: var(--fd-weight-medium);
  line-height: var(--fd-line-tight);
}

.fd-settings-row__hint {
  max-width: 420px;
  margin-top: 2px;
  color: var(--fd-text-muted);
  font-size: var(--fd-text-12);
  line-height: var(--fd-line-body);
}

.fd-settings-row__control {
  display: flex;
  flex: none;
  align-items: center;
  justify-content: flex-end;
  gap: var(--fd-space-2);
  min-width: 0;
}

.fd-settings-row--stacked .fd-settings-row__control {
  flex: 1;
  justify-content: flex-start;
  width: 100%;
}
</style>
