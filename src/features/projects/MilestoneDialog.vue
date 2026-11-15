<script setup lang="ts">
import { ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'

import FdButton from '@/components/ui/FdButton.vue'
import FdField from '@/components/ui/FdField.vue'
import FdInput from '@/components/ui/FdInput.vue'
import FdModal from '@/components/ui/FdModal.vue'
import FdSelect from '@/components/ui/FdSelect.vue'
import FdTextarea from '@/components/ui/FdTextarea.vue'
import { MILESTONE_STATUSES, milestoneStatusLabel } from '@/lib/planning'
import { usePlanningStore } from '@/stores/planning'
import type { MilestoneStatus } from '@/types/ipc'

const props = defineProps<{ projectId: string }>()
const open = defineModel<boolean>('open', { default: false })
const emit = defineEmits<{ (event: 'saved'): void }>()
const { t } = useI18n()
const planning = usePlanningStore()

const name = ref('')
const description = ref('')
const dueDate = ref('')
const status = ref<MilestoneStatus>('planned')
const error = ref('')
const saving = ref(false)
const statusOptions = MILESTONE_STATUSES.map((value) => ({
  value,
  label: milestoneStatusLabel(value, t),
}))

watch(open, (isOpen) => {
  if (!isOpen) {
    return
  }
  name.value = ''
  description.value = ''
  dueDate.value = ''
  status.value = 'planned'
  error.value = ''
  saving.value = false
})

async function submit(): Promise<void> {
  const trimmed = name.value.trim()
  if (!trimmed) {
    error.value = t('validation.required')
    return
  }
  saving.value = true
  const ok = await planning.createMilestone(props.projectId, {
    name: trimmed,
    description: description.value,
    dueDate: dueDate.value || null,
    status: status.value,
  })
  saving.value = false
  if (ok) {
    open.value = false
    emit('saved')
  }
}
</script>

<template>
  <FdModal v-model="open" :title="t('projects.addMilestone')" size="sm">
    <form class="fd-milestone-form" @submit.prevent="submit">
      <FdField :label="t('projects.milestoneName')" required :error="error">
        <template #default="{ id }">
          <FdInput :id="id" v-model="name" data-autofocus maxlength="120" :invalid="Boolean(error)" />
        </template>
      </FdField>
      <FdField :label="t('planning.description')">
        <template #default="{ id }">
          <FdTextarea :id="id" v-model="description" :rows="2" />
        </template>
      </FdField>
      <div class="fd-milestone-form__grid">
        <FdField :label="t('projects.milestoneDue')">
          <template #default="{ id }">
            <FdInput :id="id" v-model="dueDate" type="date" />
          </template>
        </FdField>
        <FdField :label="t('projects.milestoneStatus')">
          <template #default="{ id }">
            <FdSelect :id="id" v-model="status">
              <option v-for="option in statusOptions" :key="option.value" :value="option.value">
                {{ option.label }}
              </option>
            </FdSelect>
          </template>
        </FdField>
      </div>
      <div class="fd-milestone-form__footer">
        <FdButton variant="ghost" size="md" @click="open = false">
          {{ t('common.cancel') }}
        </FdButton>
        <FdButton variant="primary" size="md" type="submit" :disabled="saving">
          {{ t('common.create') }}
        </FdButton>
      </div>
    </form>
  </FdModal>
</template>

<style scoped>
.fd-milestone-form {
  display: flex;
  flex-direction: column;
  gap: var(--fd-space-3);
}

.fd-milestone-form__grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: var(--fd-space-3);
}

.fd-milestone-form__footer {
  display: flex;
  justify-content: flex-end;
  gap: var(--fd-space-2);
  padding-top: var(--fd-space-2);
  border-top: 1px solid var(--fd-border-subtle);
}
</style>
