<script setup lang="ts">
import { useI18n } from 'vue-i18n'

import FdButton from './FdButton.vue'
import FdModal from './FdModal.vue'

withDefaults(
  defineProps<{
    title: string
    body: string
    confirmLabel: string
    cancelLabel?: string
    destructive?: boolean
    busy?: boolean
  }>(),
  {
    cancelLabel: undefined,
    destructive: false,
    busy: false,
  },
)

const open = defineModel<boolean>({ default: false })
const emit = defineEmits<{ (event: 'confirm'): void }>()
const { t } = useI18n()
</script>

<template>
  <FdModal v-model="open" :title="title" size="sm">
    <p class="fd-confirm__body">{{ body }}</p>
    <template #footer>
      <FdButton variant="secondary" :disabled="busy" @click="open = false">
        {{ cancelLabel ?? t('common.cancel') }}
      </FdButton>
      <FdButton
        :variant="destructive ? 'destructive' : 'primary'"
        :loading="busy"
        @click="emit('confirm')"
      >
        {{ confirmLabel }}
      </FdButton>
    </template>
  </FdModal>
</template>

<style scoped>
.fd-confirm__body {
  color: var(--fd-text-secondary);
  font-size: var(--fd-text-13);
  line-height: var(--fd-line-body);
}
</style>
