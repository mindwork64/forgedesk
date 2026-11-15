<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'

import FdButton from '@/components/ui/FdButton.vue'
import FdModal from '@/components/ui/FdModal.vue'
import { copyText } from '@/lib/clipboard'
import { useGithubAuthStore } from '@/stores/githubAuth'
import { useToastStore } from '@/stores/toasts'

const open = defineModel<boolean>('open', { default: false })
const auth = useGithubAuthStore()
const toast = useToastStore()
const { t } = useI18n()
const copied = ref(false)

const code = computed(() => auth.flow?.userCode ?? '')
const url = computed(() => auth.flow?.verificationUri ?? '')

watch(open, (isOpen) => {
  copied.value = false
  if (isOpen) {
    void auth.start()
  } else {
    auth.cancel()
  }
})

watch(
  () => auth.connected,
  (connected) => {
    if (connected) {
      open.value = false
    }
  },
)

async function copy(value: string): Promise<void> {
  if (await copyText(value)) {
    copied.value = true
    toast.success(t('common.copied'))
    setTimeout(() => {
      copied.value = false
    }, 1500)
  }
}
</script>

<template>
  <FdModal v-model="open" :title="t('githubAuth.title')" size="sm">
    <div class="fd-github-device">
      <p class="fd-github-device__hint">{{ t('githubAuth.hint') }}</p>
      <div class="fd-github-device__code-block">
        <span>{{ t('githubAuth.code') }}</span>
        <strong>{{ code || '…' }}</strong>
        <FdButton variant="ghost" size="sm" :disabled="!code" @click="copy(code)">
          {{ copied ? t('common.copied') : t('common.copy') }}
        </FdButton>
      </div>
      <div class="fd-github-device__url">
        <span>{{ t('githubAuth.url') }}</span>
        <code>{{ url || '…' }}</code>
        <FdButton variant="ghost" size="sm" :disabled="!url" @click="copy(url)">
          {{ t('common.copy') }}
        </FdButton>
      </div>
      <p class="fd-github-device__status" role="status">
        <span v-if="auth.polling" class="fd-github-device__spinner" aria-hidden="true" />
        {{ auth.polling ? t('githubAuth.waiting') : '' }}
      </p>
      <div class="fd-github-device__footer">
        <FdButton variant="ghost" size="md" @click="open = false">
          {{ t('githubAuth.cancel') }}
        </FdButton>
      </div>
    </div>
  </FdModal>
</template>

<style scoped>
.fd-github-device {
  display: flex;
  flex-direction: column;
  gap: var(--fd-space-3);
}

.fd-github-device__hint {
  margin: 0;
  color: var(--fd-text-secondary);
  font-size: var(--fd-text-12);
  line-height: var(--fd-line-body);
}

.fd-github-device__code-block,
.fd-github-device__url {
  display: flex;
  align-items: center;
  gap: var(--fd-space-2);
  padding: 8px 10px;
  border: 1px solid var(--fd-border);
  border-radius: var(--fd-radius-control);
  background: var(--fd-bg);
}

.fd-github-device__code-block {
  flex-direction: column;
  align-items: flex-start;
}

.fd-github-device__code-block strong {
  font-family: var(--fd-font-mono);
  font-size: 24px;
  letter-spacing: 0.12em;
}

.fd-github-device__code-block span,
.fd-github-device__url span {
  color: var(--fd-text-muted);
  font-size: var(--fd-text-11);
}

.fd-github-device__url code {
  flex: 1;
  overflow: hidden;
  color: var(--fd-text-secondary);
  font-size: var(--fd-text-11);
  text-overflow: ellipsis;
  white-space: nowrap;
}

.fd-github-device__status {
  min-height: 20px;
  margin: 0;
  color: var(--fd-text-muted);
  font-size: var(--fd-text-11);
}

.fd-github-device__spinner {
  display: inline-block;
  width: 10px;
  height: 10px;
  margin-right: 6px;
  border: 1.5px solid var(--fd-border);
  border-top-color: var(--fd-accent);
  border-radius: 50%;
  animation: fd-spin 0.8s linear infinite;
}

.fd-github-device__footer {
  display: flex;
  justify-content: flex-end;
  padding-top: var(--fd-space-2);
  border-top: 1px solid var(--fd-border-subtle);
}

@keyframes fd-spin {
  to {
    transform: rotate(360deg);
  }
}
</style>
