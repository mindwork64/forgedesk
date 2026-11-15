<script setup lang="ts">
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'

import FdButton from '@/components/ui/FdButton.vue'
import FdInput from '@/components/ui/FdInput.vue'
import FdModal from '@/components/ui/FdModal.vue'
import { toIpcError } from '@/lib/ipc'
import { usePullRequestStore } from '@/stores/pullRequests'

const props = defineProps<{
  owner: string
  repo: string
  number: number
}>()
const open = defineModel<boolean>('open', { default: false })
const prs = usePullRequestStore()
const { t } = useI18n()
const issueKey = ref('')
const error = ref<string | null>(null)
const saving = ref(false)

async function submit(): Promise<void> {
  if (!issueKey.value.trim()) {
    error.value = t('prs.link.required')
    return
  }
  saving.value = true
  error.value = null
  try {
    await prs.linkIssue(props.owner, props.repo, props.number, issueKey.value.trim())
    issueKey.value = ''
    open.value = false
  } catch (caught) {
    error.value = toIpcError(caught).message
  } finally {
    saving.value = false
  }
}
</script>

<template>
  <FdModal v-model="open" :title="t('prs.link.title')" size="sm">
    <form class="fd-link-issue" @submit.prevent="submit">
      <p class="fd-link-issue__hint">{{ t('prs.link.hint') }}</p>
      <FdInput
        v-model="issueKey"
        :label="t('prs.link.issueKey')"
        placeholder="FD-123"
        autofocus
      />
      <p v-if="error" class="fd-link-issue__error" role="alert">{{ error }}</p>
      <div class="fd-link-issue__actions">
        <FdButton type="button" variant="ghost" @click="open = false">
          {{ t('common.cancel') }}
        </FdButton>
        <FdButton type="submit" variant="primary" :disabled="saving">
          {{ t('prs.link.confirm') }}
        </FdButton>
      </div>
    </form>
  </FdModal>
</template>

<style scoped>
.fd-link-issue { display: flex; flex-direction: column; gap: var(--fd-space-3); }
.fd-link-issue__hint { margin: 0; color: var(--fd-text-muted); font-size: var(--fd-text-11); line-height: var(--fd-line-body); }
.fd-link-issue__error { margin: 0; color: var(--fd-danger); font-size: var(--fd-text-11); }
.fd-link-issue__actions { display: flex; justify-content: flex-end; gap: var(--fd-space-2); padding-top: var(--fd-space-2); border-top: 1px solid var(--fd-border-subtle); }
</style>
