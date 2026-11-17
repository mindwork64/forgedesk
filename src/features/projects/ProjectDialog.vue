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
  PlanningStatus,
  Priority,
  Project,
  ProjectInput,
} from '@/types/ipc'

const props = defineProps<{
  project?: Project | null
  initiatives: Initiative[]
}>()

const open = defineModel<boolean>('open', { default: false })
const emit = defineEmits<{ (event: 'saved', project: Project): void }>()
const { t } = useI18n()
const planning = usePlanningStore()

const name = ref('')
const description = ref('')
const status = ref<PlanningStatus>('planned')
const priority = ref<Priority>('none')
const startDate = ref('')
const targetDate = ref('')
const initiativeId = ref('')
const issueKeyPrefix = ref('')
const error = ref('')
const saving = ref(false)

const priorityOptions = (['none', 'low', 'medium', 'high', 'urgent'] as Priority[]).map(
  (value) => ({ value, label: t(`priority.${value}`) }),
)
const statusOptions = PLANNING_STATUSES.map((value) => ({
  value,
  label: planningStatusLabel(value, t),
}))

watch(open, (isOpen) => {
  if (!isOpen) {
    return
  }
  const project = props.project
  name.value = project?.name ?? ''
  description.value = project?.description ?? ''
  status.value = project?.status ?? 'planned'
  priority.value = project?.priority ?? 'none'
  startDate.value = project?.startDate ?? ''
  targetDate.value = project?.targetDate ?? ''
  initiativeId.value = project?.initiative?.id ?? ''
  issueKeyPrefix.value = project?.issueKeyPrefix ?? ''
  error.value = ''
  saving.value = false
})

async function submit(): Promise<void> {
  const trimmed = name.value.trim()
  if (!trimmed) {
    error.value = t('validation.required')
    return
  }
  const input: ProjectInput = {
    name: trimmed,
    description: description.value,
    status: status.value,
    priority: priority.value,
    startDate: startDate.value || null,
    targetDate: targetDate.value || null,
    initiativeId: initiativeId.value || null,
    issueKeyPrefix: issueKeyPrefix.value.trim() || null,
  }
  saving.value = true
  const saved = props.project
    ? await planning.updateProject(props.project.id, input)
    : await planning.createProject(input)
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
    :title="project ? t('projects.editTitle') : t('projects.createTitle')"
    size="md"
  >
    <form class="fd-project-form" @submit.prevent="submit">
      <FdField :label="t('projects.name')" required :error="error">
        <template #default="{ id }">
          <FdInput
            :id="id"
            v-model="name"
            data-autofocus
            maxlength="120"
            :placeholder="t('projects.namePlaceholder')"
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
            :placeholder="t('projects.descriptionPlaceholder')"
          />
        </template>
      </FdField>

      <div class="fd-project-form__grid">
        <FdField :label="t('planning.statusLabel')">
          <template #default="{ id }">
            <FdSelect :id="id" v-model="status">
              <option
                v-for="option in statusOptions"
                :key="option.value"
                :value="option.value"
              >
                {{ option.label }}
              </option>
            </FdSelect>
          </template>
        </FdField>
        <FdField :label="t('sortMenu.priority')">
          <template #default="{ id }">
            <FdSelect :id="id" v-model="priority">
              <option
                v-for="option in priorityOptions"
                :key="option.value"
                :value="option.value"
              >
                {{ option.label }}
              </option>
            </FdSelect>
          </template>
        </FdField>
        <FdField :label="t('projects.startDate')">
          <template #default="{ id }">
            <FdInput :id="id" v-model="startDate" type="date" />
          </template>
        </FdField>
        <FdField :label="t('projects.targetDate')">
          <template #default="{ id }">
            <FdInput :id="id" v-model="targetDate" type="date" />
          </template>
        </FdField>
      </div>

      <FdField :label="t('projects.initiative')">
        <template #default="{ id }">
          <FdSelect :id="id" v-model="initiativeId">
            <option value="">{{ t('projects.noInitiative') }}</option>
            <option
              v-for="initiative in initiatives"
              :key="initiative.id"
              :value="initiative.id"
            >
              {{ initiative.name }}
            </option>
          </FdSelect>
        </template>
      </FdField>

      <FdField :label="t('projects.issueKeyPrefix')">
        <template #default="{ id }">
          <FdInput
            :id="id"
            v-model="issueKeyPrefix"
            maxlength="10"
            autocomplete="off"
            spellcheck="false"
            :placeholder="t('projects.issueKeyPrefixPlaceholder')"
          />
        </template>
      </FdField>
      <p class="fd-project-form__hint">{{ t('projects.issueKeyPrefixHint') }}</p>

      <div class="fd-project-form__footer">
        <FdButton variant="ghost" size="md" @click="open = false">
          {{ t('common.cancel') }}
        </FdButton>
        <FdButton variant="primary" size="md" type="submit" :disabled="saving">
          {{ project ? t('projects.save') : t('projects.create') }}
        </FdButton>
      </div>
    </form>
  </FdModal>
</template>

<style scoped>
.fd-project-form {
  display: flex;
  flex-direction: column;
  gap: var(--fd-space-3);
}

.fd-project-form__grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: var(--fd-space-3);
}

.fd-project-form__hint {
  margin: calc(-1 * var(--fd-space-2)) 0 0;
  color: var(--fd-text-muted);
  font-size: var(--fd-text-11);
  line-height: var(--fd-line-body);
}

.fd-project-form__footer {
  display: flex;
  justify-content: flex-end;
  gap: var(--fd-space-2);
  padding-top: var(--fd-space-2);
  border-top: 1px solid var(--fd-border-subtle);
}
</style>
