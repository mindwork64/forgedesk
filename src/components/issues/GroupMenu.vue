<script setup lang="ts">
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'

import FdButton from '@/components/ui/FdButton.vue'
import FdSelect from '@/components/ui/FdSelect.vue'

const props = defineProps<{ current: string | null }>()
const emit = defineEmits<{ (event: 'apply', groupBy: string | null): void }>()
const { t } = useI18n()

const groupBy = ref(props.current ?? '')
const options = [
  { value: '', label: t('groupMenu.none') },
  { value: 'status', label: t('filters.fields.status') },
  { value: 'priority', label: t('filters.fields.priority') },
  { value: 'project', label: t('filters.fields.project') },
  { value: 'type', label: t('filters.fields.type') },
]
</script>

<template>
  <div class="fd-group-menu">
    <p class="fd-group-menu__title">{{ t('groupMenu.title') }}</p>
    <FdSelect v-model="groupBy" :aria-label="t('groupMenu.title')">
      <option v-for="option in options" :key="option.value" :value="option.value">
        {{ option.label }}
      </option>
    </FdSelect>
    <div class="fd-group-menu__footer">
      <FdButton variant="ghost" size="sm" @click="emit('apply', null)">
        {{ t('groupMenu.none') }}
      </FdButton>
      <FdButton
        variant="primary"
        size="sm"
        @click="emit('apply', groupBy || null)"
      >
        {{ t('common.apply') }}
      </FdButton>
    </div>
  </div>
</template>

<style scoped>
.fd-group-menu {
  display: flex;
  flex-direction: column;
  gap: var(--fd-space-2);
  width: 240px;
  padding: var(--fd-space-3);
}

.fd-group-menu__title {
  margin: 0;
  font-size: var(--fd-text-12);
  font-weight: 600;
}

.fd-group-menu__footer {
  display: flex;
  justify-content: space-between;
  gap: var(--fd-space-2);
  padding-top: var(--fd-space-2);
  border-top: 1px solid var(--fd-border-subtle);
}
</style>
