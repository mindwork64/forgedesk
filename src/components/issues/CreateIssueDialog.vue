<script setup lang="ts">
import { ChevronDown, Plus } from '@lucide/vue'
import { computed, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'

import FdBadge from '@/components/ui/FdBadge.vue'
import FdButton from '@/components/ui/FdButton.vue'
import FdField from '@/components/ui/FdField.vue'
import FdInput from '@/components/ui/FdInput.vue'
import FdModal from '@/components/ui/FdModal.vue'
import FdSelect from '@/components/ui/FdSelect.vue'
import FdTextarea from '@/components/ui/FdTextarea.vue'
import { issueTypeLabel, statusLabel } from '@/lib/issue'
import { toIpcError } from '@/lib/ipc'
import { useIssueStore } from '@/stores/issues'
import type { IssueType, Priority } from '@/types/ipc'

const props = withDefaults(
  defineProps<{
    parentId?: string | null
    parentLabel?: string | null
  }>(),
  { parentId: null, parentLabel: null },
)

const open = defineModel<boolean>({ default: false })
const issueStore = useIssueStore()
const { t, te } = useI18n()

const title = ref('')
const description = ref('')
const statusId = ref('')
const priority = ref<Priority>('none')
const issueType = ref<IssueType>('task')
const projectId = ref('')
const estimate = ref('')
const startDate = ref('')
const dueDate = ref('')
const labelNames = ref('')
const moreOpen = ref(false)
const titleError = ref<string | null>(null)
const formError = ref<string | null>(null)
const busy = ref(false)

const statusOptions = computed(() =>
  issueStore.statuses.map((status) => ({
    value: status.id,
    label: statusLabel(status, t, te),
  })),
)

const priorityOptions = computed(() =>
  (['none', 'low', 'medium', 'high', 'urgent'] as Priority[]).map((value) => ({
    value,
    label: t(`priority.${value}`),
  })),
)

const typeOptions = computed(() =>
  (['bug', 'feature', 'task', 'improvement', 'chore', 'research'] as IssueType[]).map(
    (value) => ({ value, label: issueTypeLabel(value, t) }),
  ),
)

watch(
  open,
  (value) => {
    if (!value) {
      return
    }
    title.value = ''
    description.value = ''
    statusId.value = issueStore.statuses[0]?.id ?? ''
    priority.value = 'none'
    issueType.value = 'task'
    projectId.value = ''
    estimate.value = ''
    startDate.value = ''
    dueDate.value = ''
    labelNames.value = ''
    moreOpen.value = false
    titleError.value = null
    formError.value = null
  },
)

async function submit(): Promise<void> {
  if (busy.value) {
    return
  }
  titleError.value = null
  formError.value = null
  const trimmed = title.value.trim()
  if (!trimmed) {
    titleError.value = t('issues.titleRequired')
    return
  }

  const parsedEstimate = estimate.value.trim() ? Number(estimate.value) : null
  if (parsedEstimate !== null && !Number.isFinite(parsedEstimate)) {
    formError.value = t('issues.updateError')
    return
  }

  busy.value = true
  try {
    await issueStore.createIssue({
      title: trimmed,
      description: description.value,
      statusId: statusId.value || null,
      priority: priority.value,
      issueType: issueType.value,
      projectId: projectId.value || null,
      estimate: parsedEstimate,
      startDate: startDate.value || null,
      dueDate: dueDate.value || null,
      parentId: props.parentId,
      labelNames: labelNames.value
        .split(',')
        .map((value) => value.trim())
        .filter(Boolean),
    })
    open.value = false
  } catch (caught) {
    const wire = toIpcError(caught)
    formError.value = wire.message
    if (wire.field === 'title') {
      titleError.value = wire.message
    }
  } finally {
    busy.value = false
  }
}
</script>

<template>
  <FdModal v-model="open" :title="t('issues.createTitle')" size="lg">
    <form
      class="fd-issue-create"
      @submit.prevent="submit"
      @keydown.ctrl.enter.prevent="submit"
      @keydown.meta.enter.prevent="submit"
    >
      <div class="fd-issue-create__meta">
        <FdBadge tone="neutral">{{ t('issues.createHint') }}</FdBadge>
        <span v-if="parentId" class="fd-issue-create__parent">
          {{ t('issues.parent') }}: {{ parentLabel ?? parentId }}
        </span>
      </div>

      <FdField :label="t('issues.title')" :error="titleError ?? undefined" required>
        <template #default="{ id }">
          <FdInput
            :id="id"
            v-model="title"
            :invalid="!!titleError"
            :placeholder="t('issues.titlePlaceholder')"
            :maxlength="500"
            autofocus
            data-autofocus
            @update:model-value="titleError = null"
          />
        </template>
      </FdField>

      <FdField :label="t('issues.description')">
        <template #default="{ id }">
          <FdTextarea
            :id="id"
            v-model="description"
            :placeholder="t('issues.descriptionPlaceholder')"
            :rows="5"
          />
        </template>
      </FdField>

      <div class="fd-issue-create__properties">
        <FdField :label="t('issues.status')">
          <template #default="{ id }">
            <FdSelect :id="id" v-model="statusId">
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

        <FdField :label="t('issues.priority')">
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

        <FdField :label="t('issues.type')">
          <template #default="{ id }">
            <FdSelect :id="id" v-model="issueType">
              <option
                v-for="option in typeOptions"
                :key="option.value"
                :value="option.value"
              >
                {{ option.label }}
              </option>
            </FdSelect>
          </template>
        </FdField>

        <FdField :label="t('issues.project')">
          <template #default="{ id }">
            <FdSelect :id="id" v-model="projectId">
              <option value="">{{ t('issues.noProject') }}</option>
              <option
                v-for="project in issueStore.projects"
                :key="project.id"
                :value="project.id"
              >
                {{ project.name }}
              </option>
            </FdSelect>
          </template>
        </FdField>
      </div>

      <button
        type="button"
        class="fd-issue-create__more"
        :aria-expanded="moreOpen"
        @click="moreOpen = !moreOpen"
      >
        <ChevronDown
          :size="12"
          :class="{ 'fd-issue-create__more-icon--open': moreOpen }"
          aria-hidden="true"
        />
        {{ t('issues.moreProperties') }}
      </button>

      <div v-if="moreOpen" class="fd-issue-create__properties">
        <FdField :label="t('issues.labels')">
          <template #default="{ id }">
            <FdInput
              :id="id"
              v-model="labelNames"
              :placeholder="t('issues.labelsPlaceholder')"
            />
          </template>
        </FdField>

        <FdField :label="t('issues.estimate')">
          <template #default="{ id }">
            <FdInput
              :id="id"
              v-model="estimate"
              type="number"
              placeholder="—"
            />
          </template>
        </FdField>

        <FdField :label="t('issues.startDate')">
          <template #default="{ id }">
            <FdInput :id="id" v-model="startDate" type="date" />
          </template>
        </FdField>

        <FdField :label="t('issues.dueDate')">
          <template #default="{ id }">
            <FdInput :id="id" v-model="dueDate" type="date" />
          </template>
        </FdField>
      </div>

      <p v-if="formError" class="fd-issue-create__error" role="alert">
        {{ formError }}
      </p>
    </form>

    <template #footer>
      <FdButton variant="secondary" :disabled="busy" @click="open = false">
        {{ t('common.cancel') }}
      </FdButton>
      <FdButton variant="primary" :loading="busy" type="button" @click="submit">
        <Plus :size="14" aria-hidden="true" />
        {{ t('issues.create') }}
      </FdButton>
    </template>
  </FdModal>
</template>

<style scoped>
.fd-issue-create {
  display: flex;
  flex-direction: column;
  gap: var(--fd-space-4);
}

.fd-issue-create__meta {
  display: flex;
  align-items: center;
  gap: var(--fd-space-2);
  min-height: 20px;
}

.fd-issue-create__parent {
  overflow: hidden;
  color: var(--fd-text-muted);
  font-size: var(--fd-text-11);
  text-overflow: ellipsis;
  white-space: nowrap;
}

.fd-issue-create__properties {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: var(--fd-space-3);
}

.fd-issue-create__more {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  align-self: flex-start;
  padding: 3px 4px;
  border: 0;
  border-radius: 4px;
  background: transparent;
  color: var(--fd-text-muted);
  font-size: var(--fd-text-12);
}

.fd-issue-create__more:hover {
  background: var(--fd-surface-hover);
  color: var(--fd-text-secondary);
}

.fd-issue-create__more-icon--open {
  transform: rotate(180deg);
}

.fd-issue-create__error {
  padding: 8px 10px;
  border: 1px solid var(--fd-danger);
  border-radius: var(--fd-radius-control);
  background: var(--fd-danger-soft);
  color: var(--fd-danger);
  font-size: var(--fd-text-12);
}

@media (max-width: 700px) {
  .fd-issue-create__properties {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}
</style>
