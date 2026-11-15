<script setup lang="ts">
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'

import FdButton from '@/components/ui/FdButton.vue'
import FdSegmented from '@/components/ui/FdSegmented.vue'
import FdSelect from '@/components/ui/FdSelect.vue'
import type { SortDirection, SortField, SortSpec } from '@/types/ipc'

const props = defineProps<{ current: SortSpec[] }>()
const emit = defineEmits<{ (event: 'apply', sort: SortSpec[]): void }>()
const { t } = useI18n()

const field = ref<SortField>(props.current[0]?.field ?? 'updatedAt')
const direction = ref<SortDirection>(
  props.current[0]?.direction ?? 'desc',
)
const fieldOptions = [
  { value: 'updatedAt', label: t('sortMenu.updatedAt') },
  { value: 'createdAt', label: t('sortMenu.createdAt') },
  { value: 'priority', label: t('sortMenu.priority') },
  { value: 'status', label: t('sortMenu.status') },
  { value: 'title', label: t('sortMenu.titleField') },
  { value: 'dueDate', label: t('sortMenu.dueDate') },
  { value: 'estimate', label: t('sortMenu.estimate') },
]
const directionOptions = [
  { value: 'asc', label: t('sortMenu.ascending') },
  { value: 'desc', label: t('sortMenu.descending') },
]

function apply(): void {
  emit('apply', [{ field: field.value, direction: direction.value }])
}

function clear(): void {
  emit('apply', [{ field: 'updatedAt', direction: 'desc' }])
}
</script>

<template>
  <div class="fd-sort-menu">
    <p class="fd-sort-menu__title">{{ t('sortMenu.title') }}</p>
    <FdSelect v-model="field" :aria-label="t('sortMenu.title')">
      <option v-for="option in fieldOptions" :key="option.value" :value="option.value">
        {{ option.label }}
      </option>
    </FdSelect>
    <FdSegmented v-model="direction" :options="directionOptions" :label="t('sortMenu.title')" />
    <div class="fd-sort-menu__footer">
      <FdButton variant="ghost" size="sm" @click="clear">
        {{ t('sortMenu.clear') }}
      </FdButton>
      <FdButton variant="primary" size="sm" @click="apply">
        {{ t('common.apply') }}
      </FdButton>
    </div>
  </div>
</template>

<style scoped>
.fd-sort-menu {
  display: flex;
  flex-direction: column;
  gap: var(--fd-space-2);
  width: 280px;
  padding: var(--fd-space-3);
}

.fd-sort-menu__title {
  margin: 0;
  font-size: var(--fd-text-12);
  font-weight: 600;
}

.fd-sort-menu__footer {
  display: flex;
  justify-content: space-between;
  gap: var(--fd-space-2);
  padding-top: var(--fd-space-2);
  border-top: 1px solid var(--fd-border-subtle);
}
</style>
