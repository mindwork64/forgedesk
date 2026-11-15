<script setup lang="ts">
import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'

import FdButton from '@/components/ui/FdButton.vue'
import {
  countFilterConditions,
  isFilterComplete,
  pruneFilter,
  type FilterMetadata,
} from '@/lib/filter'
import type { FilterGroup, FilterNode } from '@/types/ipc'

import FilterGroupEditor from './FilterGroupEditor.vue'

const props = defineProps<{
  initial: FilterNode | null
  metadata: FilterMetadata
}>()

const emit = defineEmits<{
  (event: 'apply', filter: FilterNode | null): void
  (event: 'clear'): void
}>()
const { t } = useI18n()

function normalize(initial: FilterNode | null): FilterGroup {
  if (!initial) {
    return { kind: 'group', op: 'and', conditions: [] }
  }
  const cloned = JSON.parse(JSON.stringify(initial)) as FilterNode
  if (cloned.kind === 'group') {
    return cloned
  }
  return { kind: 'group', op: 'and', conditions: [cloned] }
}

const root = ref<FilterGroup>(normalize(props.initial))
const error = ref('')
const activeCount = computed(() => countFilterConditions(root.value))
const activeLabel = computed(() =>
  activeCount.value === 1
    ? t('filters.oneFilter')
    : t('filters.manyFilters', { count: activeCount.value }),
)
const canApply = computed(() => isFilterComplete(root.value))

function apply(): void {
  if (root.value.conditions.length === 0) {
    emit('apply', null)
    return
  }
  if (!canApply.value) {
    error.value = t('filters.incomplete')
    return
  }
  const pruned = pruneFilter(root.value)
  emit('apply', pruned)
}

function clear(): void {
  root.value = { kind: 'group', op: 'and', conditions: [] }
  error.value = ''
  emit('clear')
}
</script>

<template>
  <div class="fd-filter-builder">
    <div class="fd-filter-builder__head">
      <div>
        <p class="fd-filter-builder__title">{{ t('filters.title') }}</p>
        <p class="fd-filter-builder__count">{{ activeLabel }}</p>
      </div>
      <FdButton variant="ghost" size="sm" @click="clear">
        {{ t('filters.clear') }}
      </FdButton>
    </div>

    <FilterGroupEditor :group="root" :depth="1" :metadata="metadata" />

    <p v-if="error" class="fd-filter-builder__error" role="alert">{{ error }}</p>

    <div class="fd-filter-builder__footer">
      <FdButton variant="ghost" size="md" @click="emit('apply', null)">
        {{ t('filters.clear') }}
      </FdButton>
      <FdButton variant="primary" size="md" @click="apply">
        {{ t('filters.apply') }}
      </FdButton>
    </div>
  </div>
</template>

<style scoped>
.fd-filter-builder {
  display: flex;
  flex-direction: column;
  gap: var(--fd-space-3);
  width: min(600px, calc(100vw - 64px));
  max-height: min(560px, calc(100vh - 140px));
  overflow: auto;
  padding: var(--fd-space-3);
}

.fd-filter-builder__head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: var(--fd-space-2);
}

.fd-filter-builder__title {
  margin: 0;
  color: var(--fd-text-primary);
  font-size: var(--fd-text-13);
  font-weight: 600;
}

.fd-filter-builder__count {
  margin: 2px 0 0;
  color: var(--fd-text-muted);
  font-size: var(--fd-text-11);
}

.fd-filter-builder__error {
  margin: 0;
  color: var(--fd-danger);
  font-size: var(--fd-text-11);
}

.fd-filter-builder__footer {
  display: flex;
  justify-content: flex-end;
  gap: var(--fd-space-2);
  padding-top: var(--fd-space-2);
  border-top: 1px solid var(--fd-border-subtle);
}
</style>
