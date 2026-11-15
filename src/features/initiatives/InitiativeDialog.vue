<script setup lang="ts">
import { ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'

import FdButton from '@/components/ui/FdButton.vue'
import FdField from '@/components/ui/FdField.vue'
import FdInput from '@/components/ui/FdInput.vue'
import FdModal from '@/components/ui/FdModal.vue'
import FdSelect from '@/components/ui/FdSelect.vue'
import FdTextarea from '@/components/ui/FdTextarea.vue'
import { PLANNING_STATUSES, planningStatusLabel } from '@/lib/planning'
import { usePlanningStore } from '@/stores/planning'
import type {
  Initiative,
  InitiativeInput,
  PlanningStatus,
  Priority,
} from '@/types/ipc'

const props = defineProps<{ initiative?: Initiative | null }>()
const open = defineModel<boolean>('open', { default: false })
const emit = defineEmits<{ (event: 'saved', initiative: Initiative): void }>()
const { t } = useI18n()
const planning = usePlanningStore()

const name = ref('')
const description = ref('')
const status = ref<PlanningStatus>('planned')
const priority = ref<Priority>('none')
const startDate = ref('')
const targetDate = ref('')
const error = ref('')
const saving = ref(false)

const statusOptions = PLANNING_STATUSES.map((value) => ({
  value,
  label: planningStatusLabel(value, t),
}))
const priorityOptions = (['none', 'low', 'medium', 'high', 'urgent'] as Priority[]).map(
  (value) => ({ value, label: t(`priority.${value}`) }),
)

watch(open, (isOpen) => {
  if (!isOpen) {
    return
  }
  name.value = props.initiative?.name ?? ''
  description.value = props.initiative?.description ?? ''
  status.value = props.initiative?.status ?? 'planned'
  priority.value = props.initiative?.priority ?? 'none'
  startDate.value = props.initiative?.startDate ?? ''
  targetDate.value = props.initiative?.targetDate ?? ''
  error.value = ''
  saving.value = false
})

async function submit(): Promise<void> {
  const trimmed = name.value.trim()
  if (!trimmed) {
    error.value = t('validation.required')
    return
  }
  const input: InitiativeInput = {
    name: trimmed,
    description: description.value,
    status: status.value,
    priority: priority.value,
    startDate: startDate.value || null,
    targetDate: targetDate.value || null,
  }
  saving.value = true
  const saved = props.initiative
    ? await planning.updateInitiative(props.initiative.id, input)
    : await planning.createInitiative(input)
  saving.value = false
  if (saved) {
    open.value = false
    emit('saved', saved)
  }
}
</script>

<template>
  <FdModal
    v-model="open"
    :title="initiative ? t('initiatives.editTitle') : t('initiatives.createTitle')"
    size="md"
  >
    <form class="fd-initiative-form" @submit.prevent="submit">
      <FdField :label="t('initiatives.name')" required :error="error">
        <template #default="{ id }">
          <FdInput
            :id="id"
            v-model="name"
            data-autofocus
            maxlength="120"
            :placeholder="t('initiatives.namePlaceholder')"
            :invalid="Boolean(error)"
          />
        </template>
      </FdField>
      <FdField :label="t('planning.description')">
        <template #default="{ id }">
          <FdTextarea
            :id="id"
            v-model="description"
            :rows="3"
            :placeholder="t('initiatives.descriptionPlaceholder')"
          />
        </template>
      </FdField>
      <div class="fd-initiative-form__grid">
        <FdField :label="t('planning.statusLabel')">
          <template #default="{ id }">
            <FdSelect :id="id" v-model="status">
              <option v-for="option in statusOptions" :key="option.value" :value="option.value">
                {{ option.label }}
              </option>
            </FdSelect>
          </template>
        </FdField>
        <FdField :label="t('sortMenu.priority')">
          <template #default="{ id }">
            <FdSelect :id="id" v-model="priority">
              <option v-for="option in priorityOptions" :key="option.value" :value="option.value">
                {{ option.label }}
              </option>
            </FdSelect>
          </template>
        </FdField>
        <FdField :label="t('initiatives.startDate')">
          <template #default="{ id }">
            <FdInput :id="id" v-model="startDate" type="date" />
          </template>
        </FdField>
        <FdField :label="t('initiatives.targetDate')">
          <template #default="{ id }">
            <FdInput :id="id" v-model="targetDate" type="date" />
          </template>
        </FdField>
      </div>
      <div class="fd-initiative-form__footer">
        <FdButton variant="ghost" size="md" @click="open = false">
          {{ t('common.cancel') }}
        </FdButton>
        <FdButton variant="primary" size="md" type="submit" :disabled="saving">
          {{ initiative ? t('initiatives.save') : t('initiatives.create') }}
        </FdButton>
      </div>
    </form>
  </FdModal>
</template>

<style scoped>
.fd-initiative-form {
  display: flex;
  flex-direction: column;
  gap: var(--fd-space-3);
}

.fd-initiative-form__grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: var(--fd-space-3);
}

.fd-initiative-form__footer {
  display: flex;
  justify-content: flex-end;
  gap: var(--fd-space-2);
  padding-top: var(--fd-space-2);
  border-top: 1px solid var(--fd-border-subtle);
}
</style>
