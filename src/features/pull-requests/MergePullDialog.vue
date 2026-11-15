<script setup lang="ts">
import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'

import FdButton from '@/components/ui/FdButton.vue'
import FdModal from '@/components/ui/FdModal.vue'
import FdSegmented from '@/components/ui/FdSegmented.vue'
import { usePullRequestStore } from '@/stores/pullRequests'

const props = defineProps<{
  owner: string
  repo: string
  number: number
  headSha: string
}>()
const emit = defineEmits<{ (event: 'done'): void }>()
const open = defineModel<boolean>('open', { default: false })
const prs = usePullRequestStore()
const { t } = useI18n()
const method = ref('merge')
const saving = ref(false)
const options = computed(() => [
  { value: 'merge', label: t('prs.merge.merge') },
  { value: 'squash', label: t('prs.merge.squash') },
  { value: 'rebase', label: t('prs.merge.rebase') },
])

async function submit(): Promise<void> {
  saving.value = true
  const ok = await prs.merge(
    props.owner,
    props.repo,
    props.number,
    method.value,
    props.headSha,
  )
  saving.value = false
  if (ok) {
    open.value = false
    emit('done')
  }
}
</script>

<template>
  <FdModal v-model="open" :title="t('prs.merge.title')" size="sm">
    <div class="fd-merge">
      <p class="fd-merge__hint">{{ t('prs.merge.hint') }}</p>
      <FdSegmented v-model="method" :options="options" :label="t('prs.merge.method')" />
      <code class="fd-merge__sha">{{ headSha.slice(0, 7) }}</code>
      <div class="fd-merge__actions">
        <FdButton variant="ghost" @click="open = false">{{ t('common.cancel') }}</FdButton>
        <FdButton variant="primary" :disabled="saving" @click="submit">
          {{ saving ? t('prs.merge.merging') : t('prs.merge.confirm') }}
        </FdButton>
      </div>
    </div>
  </FdModal>
</template>

<style scoped>
.fd-merge { display: flex; flex-direction: column; gap: var(--fd-space-3); }
.fd-merge__hint { margin: 0; color: var(--fd-text-muted); font-size: var(--fd-text-11); line-height: var(--fd-line-body); }
.fd-merge__sha { color: var(--fd-text-muted); font-size: var(--fd-text-11); }
.fd-merge__actions { display: flex; justify-content: flex-end; gap: var(--fd-space-2); padding-top: var(--fd-space-2); border-top: 1px solid var(--fd-border-subtle); }
</style>
