<script setup lang="ts">
withDefaults(
  defineProps<{
    rows?: number
    height?: number
  }>(),
  { rows: 4, height: 36 },
)
</script>

<template>
  <div class="fd-skeleton" aria-busy="true" aria-hidden="true">
    <span
      v-for="row in rows"
      :key="row"
      class="fd-skeleton__row"
      :style="{ height: `${height}px` }"
    />
  </div>
</template>

<style scoped>
.fd-skeleton {
  display: flex;
  flex-direction: column;
  gap: var(--fd-space-2);
  width: 100%;
}

.fd-skeleton__row {
  border-radius: var(--fd-radius-control);
  background: var(--fd-skeleton);
  animation: fd-skeleton-pulse 1.4s ease-in-out infinite;
}

@keyframes fd-skeleton-pulse {
  0%,
  100% {
    opacity: 0.55;
  }
  50% {
    opacity: 1;
  }
}
</style>
