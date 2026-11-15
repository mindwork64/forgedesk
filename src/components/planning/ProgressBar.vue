<script setup lang="ts">
withDefaults(
  defineProps<{
    percent: number
    completed?: number
    total?: number
    showText?: boolean
  }>(),
  { completed: 0, total: 0, showText: true },
)
</script>

<template>
  <div class="fd-progress">
    <div
      class="fd-progress__track"
      role="progressbar"
      :aria-valuenow="Math.round(percent)"
      aria-valuemin="0"
      aria-valuemax="100"
    >
      <span
        class="fd-progress__fill"
        :style="{ width: `${Math.min(100, Math.max(0, percent))}%` }"
      />
    </div>
    <span v-if="showText" class="fd-progress__text">
      {{ Math.round(percent) }}%
      <span v-if="total > 0" class="fd-progress__count">
        · {{ completed }}/{{ total }}
      </span>
    </span>
  </div>
</template>

<style scoped>
.fd-progress {
  display: flex;
  align-items: center;
  gap: 8px;
  min-width: 0;
}

.fd-progress__track {
  width: 100%;
  min-width: 42px;
  height: 5px;
  overflow: hidden;
  border-radius: var(--fd-radius-pill);
  background: var(--fd-surface-selected);
}

.fd-progress__fill {
  display: block;
  height: 100%;
  border-radius: inherit;
  background: var(--fd-accent);
}

.fd-progress__text {
  flex: none;
  color: var(--fd-text-muted);
  font-size: var(--fd-text-11);
  font-variant-numeric: tabular-nums;
}

.fd-progress__count {
  opacity: 0.75;
}
</style>
