<script setup lang="ts">
import { Sparkles } from '@lucide/vue'
import { computed, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'

import PageHeader from '@/components/layout/PageHeader.vue'
import FdButton from '@/components/ui/FdButton.vue'
import FdConfirmDialog from '@/components/ui/FdConfirmDialog.vue'
import FdInput from '@/components/ui/FdInput.vue'
import FdSelect from '@/components/ui/FdSelect.vue'
import { api, toIpcError } from '@/lib/ipc'
import { copyText } from '@/lib/clipboard'
import { useToastStore } from '@/stores/toasts'
import type { AiContext, AiStatus, AiSuggestion, IssueDetail } from '@/types/ipc'

const { t } = useI18n()
const toast = useToastStore()
const status = ref<AiStatus | null>(null)
const enabled = ref(false)
const provider = ref('openai')
const baseUrl = ref('https://api.openai.com/v1')
const model = ref('gpt-4o-mini')
const apiKey = ref('')
const issueKey = ref('')
const issue = ref<IssueDetail | null>(null)
const instruction = ref('Suggest a concise acceptance checklist for this issue.')
const suggestion = ref<AiSuggestion | null>(null)
const busy = ref(false)
const error = ref<string | null>(null)
const applyOpen = ref(false)

const context = computed<AiContext | null>(() => {
  if (!issue.value) return null
  const summary = issue.value.summary
  return {
    issueKey: summary.key,
    title: summary.title,
    description: issue.value.description,
    status: summary.status.name,
    project: summary.project?.name ?? null,
    includeDescription: true,
  }
})

function applyStatus(next: AiStatus): void {
  status.value = next
  enabled.value = next.runtimeEnabled
  provider.value = next.provider
  baseUrl.value = next.baseUrl
  model.value = next.model
  apiKey.value = ''
}

async function loadStatus(): Promise<void> {
  try {
    applyStatus(await api.aiStatus())
  } catch (caught) {
    error.value = toIpcError(caught).message
  }
}

async function save(): Promise<void> {
  try {
    applyStatus(
      await api.aiConfigure({
        enabled: enabled.value,
        provider: provider.value,
        baseUrl: baseUrl.value,
        model: model.value,
        apiKey: apiKey.value || undefined,
      }),
    )
    toast.success(t('aiPage.saved'))
  } catch (caught) {
    error.value = toIpcError(caught).message
  }
}

async function loadIssue(): Promise<void> {
  error.value = null
  issue.value = null
  suggestion.value = null
  try {
    const issues = await api.listIssues({ search: issueKey.value.trim(), limit: 1 })
    if (issues.length === 0) {
      error.value = t('aiPage.issueNotFound')
      return
    }
    issue.value = await api.getIssue(issues[0]!.id)
  } catch (caught) {
    error.value = toIpcError(caught).message
  }
}

async function generate(): Promise<void> {
  if (!context.value) return
  busy.value = true
  error.value = null
  suggestion.value = null
  try {
    suggestion.value = await api.aiGenerate(context.value, instruction.value)
  } catch (caught) {
    error.value = toIpcError(caught).message
  } finally {
    busy.value = false
  }
}

async function applySuggestion(): Promise<void> {
  if (!issue.value || !suggestion.value) return
  const body = issue.value.description.trimEnd()
  const next = `${body}${body ? '\n\n' : ''}${suggestion.value.text}`.trim()
  try {
    issue.value = await api.updateIssue(issue.value.summary.id, { description: next })
    applyOpen.value = false
    toast.success(t('aiPage.applied'))
  } catch (caught) {
    error.value = toIpcError(caught).message
  }
}

async function copySuggestion(): Promise<void> {
  if (suggestion.value && (await copyText(suggestion.value.text))) {
    toast.success(t('common.copied'))
  }
}

onMounted(loadStatus)
</script>

<template>
  <div class="fd-ai-page">
    <PageHeader :title="t('pages.ai.title')" :description="t('pages.ai.description')" />
    <div class="fd-ai-content">
      <section class="fd-ai-panel">
        <h2>{{ t('aiPage.settings') }}</h2>
        <p v-if="status && !status.featureEnabled" class="fd-ai-hint">{{ t('aiPage.notBuilt') }}</p>
        <label class="fd-ai-check"><input v-model="enabled" type="checkbox" /> {{ t('aiPage.enable') }}</label>
        <label>{{ t('aiPage.provider') }}
          <FdSelect v-model="provider">
            <option value="openai">OpenAI-compatible</option>
            <option value="anthropic">Anthropic</option>
          </FdSelect>
        </label>
        <label>{{ t('aiPage.baseUrl') }}<FdInput v-model="baseUrl" /></label>
        <label>{{ t('aiPage.model') }}<FdInput v-model="model" /></label>
        <label>{{ t('aiPage.apiKey') }}<input v-model="apiKey" type="password" class="fd-ai-input" :placeholder="status?.hasKey ? t('aiPage.keyStored') : ''" /></label>
        <div class="fd-ai-actions">
          <FdButton variant="primary" @click="save">{{ t('aiPage.save') }}</FdButton>
          <FdButton v-if="status?.hasKey" variant="ghost" @click="api.aiClearKey().then(applyStatus)">{{ t('aiPage.clearKey') }}</FdButton>
        </div>
        <p class="fd-ai-hint">{{ t('aiPage.securityHint') }}</p>
        <p class="fd-ai-hint">{{ t('aiPage.mcpHint') }}</p>
      </section>

      <section class="fd-ai-panel">
        <h2>{{ t('aiPage.context') }}</h2>
        <div class="fd-ai-row">
          <FdInput v-model="issueKey" :placeholder="t('aiPage.issueKey')" />
          <FdButton variant="secondary" @click="loadIssue">{{ t('aiPage.load') }}</FdButton>
        </div>
        <template v-if="context">
          <dl class="fd-ai-context">
            <div><dt>{{ t('aiPage.issue') }}</dt><dd>{{ context.issueKey }} · {{ context.title }}</dd></div>
            <div><dt>{{ t('aiPage.status') }}</dt><dd>{{ context.status }}</dd></div>
            <div><dt>{{ t('aiPage.project') }}</dt><dd>{{ context.project ?? '—' }}</dd></div>
            <div><dt>{{ t('aiPage.descriptionIncluded') }}</dt><dd>{{ context.description ? t('common.yes') : t('common.no') }}</dd></div>
          </dl>
          <label>{{ t('aiPage.instruction') }}
            <textarea v-model="instruction" rows="3" class="fd-ai-textarea" />
          </label>
          <FdButton variant="primary" :disabled="busy || !enabled" @click="generate">
            {{ busy ? t('aiPage.generating') : t('aiPage.generate') }}
          </FdButton>
        </template>
        <p v-if="error" class="fd-ai-error" role="alert">{{ error }}</p>
      </section>

      <section v-if="suggestion" class="fd-ai-panel">
        <h2><Sparkles :size="14" /> {{ t('aiPage.suggestion') }}</h2>
        <pre class="fd-ai-suggestion">{{ suggestion.text }}</pre>
        <div class="fd-ai-actions">
          <FdButton variant="ghost" @click="copySuggestion">{{ t('common.copy') }}</FdButton>
          <FdButton variant="secondary" @click="applyOpen = true">{{ t('aiPage.apply') }}</FdButton>
        </div>
        <p class="fd-ai-hint">{{ t('aiPage.applyHint') }}</p>
      </section>
    </div>

    <FdConfirmDialog
      v-model="applyOpen"
      :title="t('aiPage.applyTitle')"
      :body="t('aiPage.applyBody')"
      :confirm-label="t('aiPage.apply')"
      @confirm="applySuggestion"
    />
  </div>
</template>

<style scoped>
.fd-ai-page { display: flex; flex-direction: column; width: 100%; height: 100%; min-width: 0; min-height: 0; overflow: hidden; }
.fd-ai-content { flex: 1; min-height: 0; overflow: auto; display: grid; grid-template-columns: 1fr 1fr; gap: var(--fd-space-3); padding: var(--fd-space-4) var(--fd-content-pad) var(--fd-space-6); align-content: start; }
.fd-ai-panel { display: flex; flex-direction: column; gap: 8px; padding: 12px; border: 1px solid var(--fd-border-subtle); border-radius: var(--fd-radius-control); background: var(--fd-surface); }
.fd-ai-panel h2 { display: flex; align-items: center; gap: 6px; margin: 0 0 4px; color: var(--fd-text-muted); font-size: var(--fd-text-11); font-weight: var(--fd-weight-medium); text-transform: uppercase; }
.fd-ai-panel label { display: flex; flex-direction: column; gap: 4px; color: var(--fd-text-secondary); font-size: var(--fd-text-11); }
.fd-ai-check { flex-direction: row !important; align-items: center; gap: 6px !important; }
.fd-ai-row, .fd-ai-actions { display: flex; gap: var(--fd-space-2); align-items: center; }
.fd-ai-context { display: grid; grid-template-columns: 1fr 1fr; gap: 6px; margin: 0; }
.fd-ai-context dt { color: var(--fd-text-muted); font-size: var(--fd-text-10); }
.fd-ai-context dd { margin: 0; color: var(--fd-text-secondary); font-size: var(--fd-text-11); }
.fd-ai-input { height: 30px; padding: 0 8px; border: 1px solid var(--fd-border); border-radius: var(--fd-radius-control); background: var(--fd-bg); color: var(--fd-text-primary); font: inherit; font-size: var(--fd-text-12); }
.fd-ai-textarea { padding: 8px; border: 1px solid var(--fd-border); border-radius: var(--fd-radius-control); background: var(--fd-bg); color: var(--fd-text-primary); font: inherit; font-size: var(--fd-text-12); resize: vertical; }
.fd-ai-hint { margin: 0; color: var(--fd-text-muted); font-size: var(--fd-text-10); line-height: var(--fd-line-body); }
.fd-ai-error { margin: 0; color: var(--fd-danger); font-size: var(--fd-text-11); }
.fd-ai-suggestion { max-height: 320px; margin: 0; overflow: auto; padding: 8px; border: 1px solid var(--fd-border-subtle); border-radius: var(--fd-radius-control); background: var(--fd-bg); color: var(--fd-text-secondary); font-family: var(--fd-font-mono); font-size: var(--fd-text-11); white-space: pre-wrap; }
@media (max-width: 1100px) { .fd-ai-content { grid-template-columns: 1fr; } }
</style>
