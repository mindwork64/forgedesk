<script setup lang="ts">
import { ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'

import FdButton from '@/components/ui/FdButton.vue'
import FdInput from '@/components/ui/FdInput.vue'
import FdModal from '@/components/ui/FdModal.vue'

const props = defineProps<{
  initialName: string
  overwriteId: string | null
}>()

const open = defineModel<boolean>('open', { default: false })
const emit = defineEmits<{
  (event: 'save', name: string, overwriteId: string | null): void
}>()
const { t } = useI18n()
const name = ref(props.initialName)
const error = ref('')

watch(open, (isOpen) => {
  if (isOpen) {
    name.value = props.initialName
    error.value = ''
  }
})

function submit(overwrite: boolean): void {
  const trimmed = name.value.trim()
  if (!trimmed) {
    error.value = t('views.name') + ' — ' + t('validation.required')
    return
  }
  emit('save', trimmed, overwrite ? props.overwriteId : null)
}
</script>

<template>
  <FdModal v-model="open" :title="t('views.save')" size="sm">
    <form class="fd-save-view" @submit.prevent="submit(true)">
      <label class="fd-save-view__field">
        <span>{{ t('views.name') }}</span>
        <FdInput
          v-model="name"
          data-autofocus
          maxlength="80"
          :placeholder="t('views.namePlaceholder')"
          :invalid="Boolean(error)"
        />
      </label>
      <p v-if="error" class="fd-save-view__error" role="alert">{{ error }}</p>

      <div class="fd-save-view__footer">
        <FdButton variant="ghost" size="md" @click="open = false">
          {{ t('common.cancel') }}
        </FdButton>
        <FdButton
          v-if="overwriteId"
          variant="secondary"
          size="md"
          @click="submit(false)"
        >
          {{ t('views.saveAsNew') }}
        </FdButton>
        <FdButton variant="primary" size="md" type="submit">
          {{ t('views.save') }}
        </FdButton>
      </div>
    </form>
  </FdModal>
</template>

<style scoped>
.fd-save-view {
  display: flex;
  flex-direction: column;
  gap: var(--fd-space-3);
}

.fd-save-view__field {
  display: flex;
  flex-direction: column;
  gap: 6px;
  color: var(--fd-text-secondary);
  font-size: var(--fd-text-12);
}

.fd-save-view__error {
  margin: 0;
  color: var(--fd-danger);
  font-size: var(--fd-text-11);
}

.fd-save-view__footer {
  display: flex;
  justify-content: flex-end;
  gap: var(--fd-space-2);
  padding-top: var(--fd-space-2);
  border-top: 1px solid var(--fd-border-subtle);
}
</style>
