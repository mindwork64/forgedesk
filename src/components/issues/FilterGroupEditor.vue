<script setup lang="ts">
import { Plus, X } from '@lucide/vue'
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'

import FdButton from '@/components/ui/FdButton.vue'
import FdIconButton from '@/components/ui/FdIconButton.vue'
import FdSegmented from '@/components/ui/FdSegmented.vue'
import { defaultCondition, type FilterMetadata } from '@/lib/filter'
import type { FilterGroup } from '@/types/ipc'

import FilterConditionRow from './FilterConditionRow.vue'

const props = defineProps<{
  group: FilterGroup
  depth: number
  metadata: FilterMetadata
}>()

const emit = defineEmits<{ (event: 'remove'): void }>()
const { t } = useI18n()

const canAddGroup = computed(
  () =>
    props.depth === 1 &&
    !props.group.conditions.some((condition) => condition.kind === 'group'),
)
const opOptions = computed(() => [
  { value: 'and', label: t('filters.all') },
  { value: 'or', label: t('filters.any') },
])

function setOp(op: string): void {
  props.group.op = op === 'or' ? 'or' : 'and'
}

function addCondition(): void {
  props.group.conditions.push(defaultCondition('status', props.metadata))
}

function addGroup(): void {
  if (!canAddGroup.value) {
    return
  }
  props.group.conditions.push({
    kind: 'group',
    op: 'and',
    conditions: [defaultCondition('priority', props.metadata)],
  })
}

function removeAt(index: number): void {
  props.group.conditions.splice(index, 1)
}
</script>

<template>
  <div class="fd-filter-group" :class="{ 'fd-filter-group--nested': depth > 1 }">
    <div class="fd-filter-group__header">
      <div class="fd-filter-group__match">
        <span class="fd-filter-group__label">{{ t('filters.match') }}</span>
        <FdSegmented
          :model-value="group.op"
          :options="opOptions"
          :label="t('filters.match')"
          @update:model-value="setOp"
        />
      </div>
      <div class="fd-filter-group__actions">
        <FdButton variant="ghost" size="sm" @click="addCondition">
          <Plus :size="12" aria-hidden="true" />
          {{ t('filters.addCondition') }}
        </FdButton>
        <FdButton
          v-if="canAddGroup"
          variant="ghost"
          size="sm"
          @click="addGroup"
        >
          {{ t('filters.addGroup') }}
        </FdButton>
        <FdIconButton
          v-if="depth > 1"
          :label="t('filters.remove')"
          @click="emit('remove')"
        >
          <X :size="13" />
        </FdIconButton>
      </div>
    </div>

    <div v-if="group.conditions.length" class="fd-filter-group__conditions">
      <template v-for="(child, index) in group.conditions" :key="index">
        <FilterConditionRow
          v-if="child.kind === 'condition'"
          :condition="child"
          :metadata="metadata"
          @remove="removeAt(index)"
        />
        <FilterGroupEditor
          v-else
          :group="child"
          :depth="depth + 1"
          :metadata="metadata"
          @remove="removeAt(index)"
        />
      </template>
    </div>
    <p v-else class="fd-filter-group__empty">{{ t('filters.empty') }}</p>
  </div>
</template>

<style scoped>
.fd-filter-group {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.fd-filter-group--nested {
  padding: 8px;
  border-left: 2px solid var(--fd-accent-soft);
  border-radius: var(--fd-radius-control);
  background: var(--fd-surface);
}

.fd-filter-group__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--fd-space-2);
  flex-wrap: wrap;
}

.fd-filter-group__match {
  display: flex;
  align-items: center;
  gap: 6px;
}

.fd-filter-group__label {
  color: var(--fd-text-muted);
  font-size: var(--fd-text-11);
}

.fd-filter-group__actions {
  display: flex;
  align-items: center;
  gap: 2px;
}

.fd-filter-group__conditions {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.fd-filter-group__empty {
  color: var(--fd-text-muted);
  font-size: var(--fd-text-11);
}
</style>
