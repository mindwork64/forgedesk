<script setup lang="ts">
import { ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'

import FdButton from '@/components/ui/FdButton.vue'
import FdField from '@/components/ui/FdField.vue'
import FdInput from '@/components/ui/FdInput.vue'
import FdModal from '@/components/ui/FdModal.vue'
import FdSelect from '@/components/ui/FdSelect.vue'
import { useGitStore } from '@/stores/git'
import type { Project, Repository } from '@/types/ipc'

const props = defineProps<{ projects: Project[] }>()
const open = defineModel<boolean>('open', { default: false })
const emit = defineEmits<{ (event: 'saved', repository: Repository): void }>()
const { t } = useI18n()
const git = useGitStore()

const path = ref('')
const name = ref('')
const projectId = ref('')
const error = ref('')
const saving = ref(false)

watch(open, (isOpen) => {
  if (!isOpen) {
    return
  }
  path.value = ''
  name.value = ''
  projectId.value = ''
  error.value = ''
  saving.value = false
})

async function submit(): Promise<void> {
  if (!path.value.trim()) {
    error.value = t('validation.required')
    return
  }
  saving.value = true
  const repository = await git.addRepository({
    path: path.value.trim(),
    name: name.value.trim() || null,
    projectId: projectId.value || null,
  })
  saving.value = false
  if (repository) {
    open.value = false
    emit('saved', repository)
  }
}
</script>

<template>
  <FdModal v-model="open" :title="t('repositories.addTitle')" size="md">
    <form class="fd-repository-form" @submit.prevent="submit">
      <FdField :label="t('repositories.path')" required :error="error">
        <template #default="{ id }">
          <FdInput
            :id="id"
            v-model="path"
            data-autofocus
            :placeholder="t('repositories.pathPlaceholder')"
            :invalid="Boolean(error)"
          />
        </template>
      </FdField>
      <FdField :label="t('repositories.name')">
        <template #default="{ id }">
          <FdInput
            :id="id"
            v-model="name"
            maxlength="160"
            :placeholder="t('repositories.namePlaceholder')"
          />
        </template>
      </FdField>
      <FdField :label="t('repositories.project')">
        <template #default="{ id }">
          <FdSelect :id="id" v-model="projectId">
            <option value="">{{ t('repositories.noProject') }}</option>
            <option v-for="project in projects" :key="project.id" :value="project.id">
              {{ project.name }}
            </option>
          </FdSelect>
        </template>
      </FdField>
      <p class="fd-repository-form__hint">{{ t('integrations.localHint') }}</p>
      <div class="fd-repository-form__footer">
        <FdButton variant="ghost" size="md" @click="open = false">
          {{ t('common.cancel') }}
        </FdButton>
        <FdButton variant="primary" size="md" type="submit" :disabled="saving">
          {{ t('repositories.add') }}
        </FdButton>
      </div>
    </form>
  </FdModal>
</template>

<style scoped>
.fd-repository-form {
  display: flex;
  flex-direction: column;
  gap: var(--fd-space-3);
}

.fd-repository-form__hint {
  margin: 0;
  color: var(--fd-text-muted);
  font-size: var(--fd-text-11);
}

.fd-repository-form__footer {
  display: flex;
  justify-content: flex-end;
  gap: var(--fd-space-2);
  padding-top: var(--fd-space-2);
  border-top: 1px solid var(--fd-border-subtle);
}
</style>
