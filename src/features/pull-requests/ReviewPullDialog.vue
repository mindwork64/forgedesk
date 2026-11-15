<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'

import FdButton from '@/components/ui/FdButton.vue'
import FdModal from '@/components/ui/FdModal.vue'
import FdSegmented from '@/components/ui/FdSegmented.vue'
import { usePullRequestStore } from '@/stores/pullRequests'

const props = defineProps<{
  owner: string
  repo: string
  number: number
  isAuthor: boolean
}>()
const emit = defineEmits<{ (event: 'done'): void }>()
const open = defineModel<boolean>('open', { default: false })
const prs = usePullRequestStore()
const { t } = useI18n()
const event = ref('COMMENT')
const body = ref('')
const saving = ref(false)

const options = computed(() =>
  [
    { value: 'APPROVE', label: t('prs.review.approve') },
    { value: 'REQUEST_CHANGES', label: t('prs.review.requestChanges') },
    { value: 'COMMENT', label: t('prs.review.comment') },
  ].filter((option) => option.value !== 'APPROVE' || !props.isAuthor),
)

watch(open, (isOpen) => {
  if (isOpen) {
    event.value = props.isAuthor ? 'COMMENT' : 'APPROVE'
    body.value = ''
  }
})

async function submit(): Promise<void> {
  saving.value = true
  const ok = await prs.submitReview(
    props.owner,
    props.repo,
    props.number,
    event.value,
    body.value,
  )
  saving.value = false
  if (ok) {
    open.value = false
    emit('done')
  }
}
</script>

<template>
  <FdModal v-model="open" :title="t('prs.review.title')" size="sm">
    <form class="fd-review-form" @submit.prevent="submit">
      <FdSegmented v-model="event" :options="options" :label="t('prs.review.event')" />
      <textarea
        v-model="body"
        class="fd-review-form__body"
        rows="5"
        :placeholder="t('prs.review.bodyPlaceholder')"
        :aria-label="t('prs.review.body')"
      />
      <p class="fd-review-form__hint">{{ t('prs.review.hint') }}</p>
      <div class="fd-review-form__actions">
        <FdButton type="button" variant="ghost" @click="open = false">{{ t('common.cancel') }}</FdButton>
        <FdButton type="submit" variant="primary" :disabled="saving">
          {{ saving ? t('prs.review.submitting') : t('prs.review.confirm') }}
        </FdButton>
      </div>
    </form>
  </FdModal>
</template>

<style scoped>
.fd-review-form { display: flex; flex-direction: column; gap: var(--fd-space-3); }
.fd-review-form__body { min-height: 96px; padding: 8px; border: 1px solid var(--fd-border); border-radius: var(--fd-radius-control); background: var(--fd-bg); color: var(--fd-text-primary); font: inherit; font-size: var(--fd-text-12); resize: vertical; }
.fd-review-form__body:focus { border-color: var(--fd-accent); outline: none; }
.fd-review-form__hint { margin: 0; color: var(--fd-text-muted); font-size: var(--fd-text-11); line-height: var(--fd-line-body); }
.fd-review-form__actions { display: flex; justify-content: flex-end; gap: var(--fd-space-2); padding-top: var(--fd-space-2); border-top: 1px solid var(--fd-border-subtle); }
</style>
