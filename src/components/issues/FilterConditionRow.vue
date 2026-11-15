<script setup lang="ts">
import { X } from '@lucide/vue'
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'

import FdIconButton from '@/components/ui/FdIconButton.vue'
import FdInput from '@/components/ui/FdInput.vue'
import FdSelect from '@/components/ui/FdSelect.vue'
import {
  defaultCondition,
  fieldLabel,
  FILTER_FIELDS,
  operatorLabel,
  operatorsFor,
  type FilterMetadata,
} from '@/lib/filter'
import type {
  FilterCondition,
  FilterField,
  FilterOperator,
} from '@/types/ipc'

const props = defineProps<{
  condition: FilterCondition
  metadata: FilterMetadata
}>()

const emit = defineEmits<{ (event: 'remove'): void }>()
const { t, te } = useI18n()

const fieldOptions = computed(() =>
  FILTER_FIELDS.map((field) => ({
    value: field,
    label: fieldLabel(field, t, te),
  })),
)
const operatorOptions = computed(() =>
  operatorsFor(props.condition.field).map((operator) => ({
    value: operator,
    label: operatorLabel(operator, t),
  })),
)
const statusOptions = computed(() =>
  props.metadata.statuses.map((status) => ({
    value: status.id,
    label: status.name,
  })),
)
const categoryOptions = computed(() =>
  (['backlog', 'unstarted', 'started', 'completed', 'canceled'] as const).map(
    (value) => ({ value, label: t(`filters.categories.${value}`) }),
  ),
)
const priorityOptions = computed(() =>
  (['none', 'low', 'medium', 'high', 'urgent'] as const).map((value) => ({
    value,
    label: t(`priority.${value}`),
  })),
)
const typeOptions = computed(() =>
  (['bug', 'feature', 'task', 'improvement', 'chore', 'research'] as const).map(
    (value) => ({ value, label: t(`issueType.${value}`) }),
  ),
)
const projectOptions = computed(() =>
  props.metadata.projects.map((project) => ({
    value: project.id,
    label: project.name,
  })),
)
const labelOptions = computed(() =>
  props.metadata.labels.map((label) => ({
    value: label.id,
    label: label.name,
  })),
)
const issueOptions = computed(() =>
  props.metadata.issues.map((issue) => ({
    value: issue.id,
    label: `${issue.key} · ${issue.title}`,
  })),
)
const sourceOptions = computed(() =>
  (['local', 'github', 'gitlab', 'gitea'] as const).map((value) => ({
    value,
    label: t(`filters.sources.${value}`),
  })),
)

const stringValue = computed(() =>
  typeof props.condition.value === 'string' ? props.condition.value : '',
)
const numberValue = computed(() =>
  typeof props.condition.value === 'number' ? String(props.condition.value) : '',
)
const booleanValue = computed(() =>
  props.condition.value === true ? 'true' : 'false',
)

function applyField(field: string): void {
  const next = defaultCondition(field as FilterField, props.metadata)
  props.condition.field = next.field
  props.condition.operator = next.operator
  props.condition.value = next.value
}

function applyOperator(operator: string): void {
  props.condition.operator = operator as FilterOperator
  if (operator === 'isEmpty' || operator === 'isNotEmpty') {
    props.condition.value = undefined
    return
  }
  if (props.condition.value === undefined || props.condition.value === '') {
    props.condition.value = defaultCondition(
      props.condition.field,
      props.metadata,
    ).value
  }
}

function applyString(value: string): void {
  props.condition.value = value
}

function applyNumber(value: string): void {
  const parsed = Number(value)
  props.condition.value = Number.isFinite(parsed) ? parsed : ''
}

function applyBoolean(value: string): void {
  props.condition.value = value === 'true'
}
</script>

<template>
  <div class="fd-filter-row">
    <FdSelect
      :model-value="condition.field"
      :aria-label="t('filters.field')"
      class="fd-filter-row__field"
      @update:model-value="applyField"
    >
      <option
        v-for="option in fieldOptions"
        :key="option.value"
        :value="option.value"
      >
        {{ option.label }}
      </option>
    </FdSelect>

    <FdSelect
      :model-value="condition.operator"
      :aria-label="t('filters.operator')"
      class="fd-filter-row__operator"
      @update:model-value="applyOperator"
    >
      <option
        v-for="option in operatorOptions"
        :key="option.value"
        :value="option.value"
      >
        {{ option.label }}
      </option>
    </FdSelect>

    <div class="fd-filter-row__value">
      <template v-if="condition.operator === 'isEmpty' || condition.operator === 'isNotEmpty'">
        <span class="fd-filter-row__empty-value">—</span>
      </template>
      <FdSelect
        v-else-if="condition.field === 'status'"
        :model-value="stringValue"
        :aria-label="t('filters.value')"
        @update:model-value="applyString"
      >
        <option value="">{{ t('common.none') }}</option>
        <option v-for="option in statusOptions" :key="option.value" :value="option.value">
          {{ option.label }}
        </option>
      </FdSelect>
      <FdSelect
        v-else-if="condition.field === 'statusCategory'"
        :model-value="stringValue"
        :aria-label="t('filters.value')"
        @update:model-value="applyString"
      >
        <option v-for="option in categoryOptions" :key="option.value" :value="option.value">
          {{ option.label }}
        </option>
      </FdSelect>
      <FdSelect
        v-else-if="condition.field === 'priority'"
        :model-value="stringValue"
        :aria-label="t('filters.value')"
        @update:model-value="applyString"
      >
        <option v-for="option in priorityOptions" :key="option.value" :value="option.value">
          {{ option.label }}
        </option>
      </FdSelect>
      <FdSelect
        v-else-if="condition.field === 'type'"
        :model-value="stringValue"
        :aria-label="t('filters.value')"
        @update:model-value="applyString"
      >
        <option v-for="option in typeOptions" :key="option.value" :value="option.value">
          {{ option.label }}
        </option>
      </FdSelect>
      <FdSelect
        v-else-if="condition.field === 'project'"
        :model-value="stringValue"
        :aria-label="t('filters.value')"
        @update:model-value="applyString"
      >
        <option value="">{{ t('common.none') }}</option>
        <option v-for="option in projectOptions" :key="option.value" :value="option.value">
          {{ option.label }}
        </option>
      </FdSelect>
      <FdSelect
        v-else-if="condition.field === 'label'"
        :model-value="stringValue"
        :aria-label="t('filters.value')"
        @update:model-value="applyString"
      >
        <option value="">{{ t('common.none') }}</option>
        <option v-for="option in labelOptions" :key="option.value" :value="option.value">
          {{ option.label }}
        </option>
      </FdSelect>
      <FdSelect
        v-else-if="condition.field === 'parent'"
        :model-value="stringValue"
        :aria-label="t('filters.value')"
        @update:model-value="applyString"
      >
        <option value="">{{ t('common.none') }}</option>
        <option v-for="option in issueOptions" :key="option.value" :value="option.value">
          {{ option.label }}
        </option>
      </FdSelect>
      <FdSelect
        v-else-if="condition.field === 'source'"
        :model-value="stringValue"
        :aria-label="t('filters.value')"
        @update:model-value="applyString"
      >
        <option v-for="option in sourceOptions" :key="option.value" :value="option.value">
          {{ option.label }}
        </option>
      </FdSelect>
      <FdSelect
        v-else-if="condition.field === 'archived'"
        :model-value="booleanValue"
        :aria-label="t('filters.value')"
        @update:model-value="applyBoolean"
      >
        <option value="true">{{ t('filters.archivedTrue') }}</option>
        <option value="false">{{ t('filters.archivedFalse') }}</option>
      </FdSelect>
      <FdInput
        v-else-if="condition.field === 'estimate'"
        :model-value="numberValue"
        type="number"
        :aria-label="t('filters.value')"
        @update:model-value="applyNumber"
      />
      <FdInput
        v-else-if="
          condition.field === 'startDate' ||
          condition.field === 'dueDate' ||
          condition.field === 'createdAt' ||
          condition.field === 'updatedAt'
        "
        :model-value="stringValue"
        type="date"
        :aria-label="t('filters.value')"
        @update:model-value="applyString"
      />
      <FdInput
        v-else
        :model-value="stringValue"
        :aria-label="t('filters.value')"
        :placeholder="t('filters.value')"
        @update:model-value="applyString"
      />
    </div>

    <FdIconButton :label="t('filters.remove')" @click="emit('remove')">
      <X :size="13" />
    </FdIconButton>
  </div>
</template>

<style scoped>
.fd-filter-row {
  display: grid;
  grid-template-columns: minmax(110px, 0.9fr) minmax(96px, 0.7fr) minmax(130px, 1.2fr) 28px;
  gap: 6px;
  align-items: center;
}

.fd-filter-row__empty-value {
  display: flex;
  align-items: center;
  height: var(--fd-input-height);
  padding-left: 4px;
  color: var(--fd-text-muted);
  font-size: var(--fd-text-12);
}

@media (max-width: 760px) {
  .fd-filter-row {
    grid-template-columns: 1fr 1fr;
  }

  .fd-filter-row__value {
    grid-column: 1 / 2;
  }
}
</style>
