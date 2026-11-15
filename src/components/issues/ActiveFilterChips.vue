<script setup lang="ts">
import { X } from '@lucide/vue'
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'

import {
  countFilterConditions,
  filterChipLabel,
  type FilterMetadata,
} from '@/lib/filter'
import type { FilterNode } from '@/types/ipc'

const props = defineProps<{
  filter: FilterNode
  metadata: FilterMetadata
}>()

const emit = defineEmits<{
  (event: 'remove', index: number): void
  (event: 'clear'): void
}>()
const { t, te } = useI18n()

const topLevel = computed(() =>
  props.filter.kind === 'group' ? props.filter.conditions : [props.filter],
)
const count = computed(() => countFilterConditions(props.filter))

function label(node: FilterNode): string {
  if (node.kind === 'condition') {
    return filterChipLabel(node, props.metadata, t, te)
  }
  return t('filters.groupChip', {
    count: countFilterConditions(node),
  })
}
</script>

<template>
  <div class="fd-filter-chips" role="list" :aria-label="t('filters.title')">
    <button
      v-for="(node, index) in topLevel"
      :key="index"
      type="button"
      class="fd-filter-chip"
      role="listitem"
      @click="emit('remove', index)"
    >
      <span>{{ label(node) }}</span>
      <X :size="11" aria-hidden="true" />
    </button>
    <button type="button" class="fd-filter-chips__clear" @click="emit('clear')">
      {{ t('filters.clear') }}
      <span aria-hidden="true">·</span>
      {{ count }}
    </button>
  </div>
</template>

<style scoped>
.fd-filter-chips {
  display: flex;
  align-items: center;
  gap: 6px;
  flex-wrap: wrap;
  padding: 0 var(--fd-content-pad) var(--fd-space-2);
}

.fd-filter-chip {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  max-width: 280px;
  height: 24px;
  padding: 0 7px;
  border: 1px solid var(--fd-border);
  border-radius: var(--fd-radius-pill);
  background: var(--fd-surface-raised);
  color: var(--fd-text-secondary);
  font-size: var(--fd-text-11);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.fd-filter-chip:hover {
  color: var(--fd-danger);
  border-color: var(--fd-danger);
}

.fd-filter-chips__clear {
  height: 24px;
  padding: 0 4px;
  border: 0;
  background: transparent;
  color: var(--fd-text-muted);
  font-size: var(--fd-text-11);
  cursor: pointer;
}

.fd-filter-chips__clear:hover {
  color: var(--fd-accent);
}
</style>
