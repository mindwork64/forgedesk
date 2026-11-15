<script setup lang="ts">
import { onMounted, ref } from 'vue'

withDefaults(defineProps<{ label?: string }>(), { label: undefined })

const emit = defineEmits<{ (event: 'close'): void }>()
const panel = ref<HTMLElement | null>(null)

onMounted(() => {
  panel.value?.focus()
})

function onKeydown(event: KeyboardEvent): void {
  if (event.key === 'Escape') {
    event.stopPropagation()
    emit('close')
  }
}
</script>

<template>
  <aside
    ref="panel"
    class="fd-entity-pane"
    :aria-label="label"
    tabindex="-1"
    data-testid="detail-pane"
    @keydown="onKeydown"
  >
    <slot />
  </aside>
</template>

<style scoped>
.fd-entity-pane {
  position: relative;
  display: flex;
  flex: 0 0 var(--fd-detail-width);
  flex-direction: column;
  width: var(--fd-detail-width);
  min-height: 0;
  overflow: hidden;
  border-left: 1px solid var(--fd-border);
  background: var(--fd-surface);
  outline: none;
}

@media (max-width: 1279px) {
  .fd-entity-pane {
    position: absolute;
    top: 0;
    right: 0;
    bottom: 0;
    z-index: 30;
    width: min(var(--fd-detail-width), 100%);
    border-left: 1px solid var(--fd-border);
    box-shadow: var(--fd-shadow-modal);
  }
}
</style>
