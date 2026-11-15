<script setup lang="ts">
defineProps<{ keys: string[] }>()
</script>

<template>
  <span class="fd-kbd-group">
    <template v-for="(key, index) in keys" :key="`${key}-${index}`">
      <span v-if="index > 0" class="fd-kbd-plus" aria-hidden="true">+</span>
      <kbd class="fd-kbd">{{ key }}</kbd>
    </template>
  </span>
</template>

<style scoped>
.fd-kbd-group {
  display: inline-flex;
  align-items: center;
  gap: 3px;
}

.fd-kbd {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 18px;
  height: 18px;
  padding: 0 4px;
  border: 1px solid var(--fd-border);
  border-radius: 4px;
  background: var(--fd-surface);
  color: var(--fd-text-secondary);
  font-family: var(--fd-font-sans);
  font-size: var(--fd-text-11);
  font-weight: var(--fd-weight-medium);
  line-height: 1;
}

.fd-kbd-plus {
  color: var(--fd-text-muted);
  font-size: var(--fd-text-11);
}
</style>
