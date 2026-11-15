<script setup lang="ts">
import { ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'

import FdButton from '@/components/ui/FdButton.vue'
import FdField from '@/components/ui/FdField.vue'
import FdInput from '@/components/ui/FdInput.vue'
import FdModal from '@/components/ui/FdModal.vue'
import { dateOffsetIso, todayIso } from '@/lib/planning'
import { usePlanningStore } from '@/stores/planning'
import type { Cycle, CycleInput } from '@/types/ipc'

const props = defineProps<{ cycle?: Cycle | null }>()
const open = defineModel<boolean>('open', { default: false })
const emit = defineEmits<{ (event: 'saved', cycle: Cycle): void }>()
const { t } = useI18n()
const planning = usePlanningStore()

const name = ref('')
const startDate = ref(todayIso())
const endDate = ref(dateOffsetIso(14))
const error = ref('')
const saving = ref(false)

watch(open, (isOpen) => {
  if (!isOpen) {
    return
  }
  name.value = props.cycle?.name ?? ''
  startDate.value = props.cycle?.startDate ?? todayIso()
  endDate.value = props.cycle?.endDate ?? dateOffsetIso(14)
  error.value = ''
  saving.value = false
})

async function submit(): Promise<void> {
  const trimmed = name.value.trim()
  if (!trimmed) {
    error.value = t('validation.required')
    return
  }
  const input: CycleInput = {
    name: trimmed,
    startDate: startDate.value,
    endDate: endDate.value,
  }
  saving.value = true
  const saved = props.cycle
    ? await planning.updateCycle(props.cycle.id, input)
    : await planning.createCycle(input)
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
    :title="cycle ? t('cycles.editTitle') : t('cycles.createTitle')"
    size="sm"
  >
    <form class="fd-cycle-form" @submit.prevent="submit">
      <FdField :label="t('cycles.name')" required :error="error">
        <template #default="{ id }">
          <FdInput
            :id="id"
            v-model="name"
            data-autofocus
            maxlength="120"
            :placeholder="t('cycles.namePlaceholder')"
            :invalid="Boolean(error)"
          />
        </template>
      </FdField>
      <div class="fd-cycle-form__grid">
        <FdField :label="t('cycles.startDate')">
          <template #default="{ id }">
            <FdInput :id="id" v-model="startDate" type="date" />
          </template>
        </FdField>
        <FdField :label="t('cycles.endDate')">
          <template #default="{ id }">
            <FdInput :id="id" v-model="endDate" type="date" />
          </template>
        </FdField>
      </div>
      <div class="fd-cycle-form__footer">
        <FdButton variant="ghost" size="md" @click="open = false">
          {{ t('common.cancel') }}
        </FdButton>
        <FdButton variant="primary" size="md" type="submit" :disabled="saving">
          {{ cycle ? t('cycles.save') : t('cycles.create') }}
        </FdButton>
      </div>
    </form>
  </FdModal>
</template>

<style scoped>
.fd-cycle-form {
  display: flex;
  flex-direction: column;
  gap: var(--fd-space-3);
}

.fd-cycle-form__grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: var(--fd-space-3);
}

.fd-cycle-form__footer {
  display: flex;
  justify-content: flex-end;
  gap: var(--fd-space-2);
  padding-top: var(--fd-space-2);
  border-top: 1px solid var(--fd-border-subtle);
}
</style>
