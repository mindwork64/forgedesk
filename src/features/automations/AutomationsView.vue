<script setup lang="ts">
import { Zap } from '@lucide/vue'
import { computed, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'

import PageHeader from '@/components/layout/PageHeader.vue'
import FdButton from '@/components/ui/FdButton.vue'
import FdConfirmDialog from '@/components/ui/FdConfirmDialog.vue'
import FdEmptyState from '@/components/ui/FdEmptyState.vue'
import FdErrorState from '@/components/ui/FdErrorState.vue'
import FdInput from '@/components/ui/FdInput.vue'
import FdLoadingSkeleton from '@/components/ui/FdLoadingSkeleton.vue'
import FdModal from '@/components/ui/FdModal.vue'
import FdSelect from '@/components/ui/FdSelect.vue'
import { formatRelative } from '@/lib/format'
import { useAppStore } from '@/stores/app'
import { useAutomationStore } from '@/stores/automations'
import type { AutomationRule } from '@/types/ipc'

const automations = useAutomationStore()
const app = useAppStore()
const { t } = useI18n()
const editorOpen = ref(false)
const previewOpen = ref(false)
const historyOpen = ref(false)
const deleteOpen = ref(false)
const active = ref<AutomationRule | null>(null)
const name = ref('')
const trigger = ref('issue_created')
const label = ref('automated')

const triggers = computed(() => [
  { value: 'issue_created', label: t('automations.triggerCreated') },
  { value: 'status_changed', label: t('automations.triggerStatus') },
])

function openEditor(): void {
  name.value = ''
  trigger.value = 'issue_created'
  label.value = 'automated'
  editorOpen.value = true
}

async function save(): Promise<void> {
  const ok = await automations.create({
    name: name.value,
    triggerEvent: trigger.value,
    actionValue: label.value,
    enabled: true,
  })
  if (ok) editorOpen.value = false
}

async function showPreview(rule: AutomationRule): Promise<void> {
  active.value = rule
  await automations.loadPreview(rule)
  previewOpen.value = true
}

async function showHistory(rule: AutomationRule): Promise<void> {
  active.value = rule
  await automations.loadHistory(rule)
  historyOpen.value = true
}

function askDelete(rule: AutomationRule): void {
  active.value = rule
  deleteOpen.value = true
}

onMounted(() => {
  void automations.load()
})
</script>

<template>
  <div class="fd-auto-page">
    <PageHeader :title="t('pages.automations.title')" :description="t('pages.automations.description')">
      <template #actions>
        <FdButton variant="primary" @click="openEditor">{{ t('automations.newRule') }}</FdButton>
      </template>
    </PageHeader>

    <div class="fd-auto-content">
      <div v-if="automations.loading" class="fd-auto-state">
        <FdLoadingSkeleton :rows="5" :height="52" />
      </div>
      <div v-else-if="automations.error" class="fd-auto-state">
        <FdErrorState
          :title="t('automations.loadError')"
          :message="automations.error.message"
          :code="automations.error.code"
          :detail="automations.error.detail"
          @retry="automations.load()"
        />
      </div>
      <FdEmptyState
        v-else-if="automations.rules.length === 0"
        :icon="Zap"
        :title="t('automations.empty')"
        :body="t('automations.emptyBody')"
      >
        <FdButton variant="primary" @click="openEditor">{{ t('automations.newRule') }}</FdButton>
      </FdEmptyState>
      <div v-else class="fd-auto-table" role="table" :aria-label="t('pages.automations.title')">
        <div class="fd-auto-grid fd-auto-grid--head">
          <span>{{ t('automations.enabled') }}</span>
          <span>{{ t('automations.rule') }}</span>
          <span>{{ t('automations.action') }}</span>
          <span>{{ t('automations.lastRun') }}</span>
          <span />
        </div>
        <div v-for="rule in automations.rules" :key="rule.id" class="fd-auto-grid fd-auto-row">
          <button
            type="button"
            class="fd-auto-toggle"
            :class="{ 'fd-auto-toggle--on': rule.enabled }"
            :aria-pressed="rule.enabled"
            @click="automations.toggle(rule)"
          >
            {{ rule.enabled ? t('automations.on') : t('automations.off') }}
          </button>
          <span class="fd-auto-name">
            <strong>{{ rule.name }}</strong>
            <small>{{ t(`automations.trigger_${rule.triggerEvent}`) }}</small>
          </span>
          <span>{{ rule.actionValue }}</span>
          <span class="fd-auto-muted">
            <template v-if="rule.lastRunAt">{{ formatRelative(rule.lastRunAt, app.resolvedLocale) }} · {{ rule.lastResult }}</template>
            <template v-else>{{ t('automations.never') }}</template>
          </span>
          <span class="fd-auto-actions">
            <FdButton variant="ghost" size="sm" @click="showPreview(rule)">{{ t('automations.preview') }}</FdButton>
            <FdButton variant="ghost" size="sm" @click="automations.run(rule)">{{ t('automations.run') }}</FdButton>
            <FdButton variant="ghost" size="sm" @click="showHistory(rule)">{{ t('automations.history') }}</FdButton>
            <FdButton variant="ghost" size="sm" @click="askDelete(rule)">{{ t('automations.delete') }}</FdButton>
          </span>
        </div>
      </div>
    </div>

    <FdModal v-model="editorOpen" :title="t('automations.newRule')" size="sm">
      <form class="fd-auto-form" @submit.prevent="save">
        <label>{{ t('automations.name') }}<FdInput v-model="name" autofocus /></label>
        <label>
          {{ t('automations.trigger') }}
          <FdSelect v-model="trigger">
            <option v-for="option in triggers" :key="option.value" :value="option.value">{{ option.label }}</option>
          </FdSelect>
        </label>
        <label>{{ t('automations.label') }}<FdInput v-model="label" /></label>
        <p class="fd-auto-hint">{{ t('automations.localOnly') }}</p>
        <div class="fd-auto-form__actions">
          <FdButton type="button" variant="ghost" @click="editorOpen = false">{{ t('common.cancel') }}</FdButton>
          <FdButton type="submit" variant="primary" :disabled="automations.busy">{{ t('automations.create') }}</FdButton>
        </div>
      </form>
    </FdModal>

    <FdModal v-model="previewOpen" :title="t('automations.previewTitle')" size="md">
      <p class="fd-auto-hint">{{ t('automations.previewHint') }}</p>
      <p v-if="automations.preview.length === 0" class="fd-auto-hint">{{ t('automations.noMatches') }}</p>
      <div v-for="event in automations.preview" :key="event.eventId" class="fd-auto-event">
        <strong>{{ event.entityKey }}</strong> {{ event.entityTitle }}
      </div>
    </FdModal>

    <FdModal v-model="historyOpen" :title="t('automations.historyTitle')" size="md">
      <p v-if="automations.history.length === 0" class="fd-auto-hint">{{ t('automations.noRuns') }}</p>
      <div v-for="run in automations.history" :key="run.id" class="fd-auto-run">
        <span>{{ formatRelative(run.createdAt, app.resolvedLocale) }}</span>
        <strong>{{ run.status }}</strong>
        <span>{{ run.message }}</span>
      </div>
    </FdModal>

    <FdConfirmDialog
      v-model="deleteOpen"
      :title="t('automations.deleteTitle')"
      :body="t('automations.deleteBody')"
      :confirm-label="t('automations.delete')"
      destructive
      @confirm="active && automations.remove(active)"
    />
  </div>
</template>

<style scoped>
.fd-auto-page { display: flex; flex-direction: column; width: 100%; height: 100%; min-width: 0; min-height: 0; overflow: hidden; }
.fd-auto-content { flex: 1; min-height: 0; overflow: auto; }
.fd-auto-state { padding: var(--fd-space-5) var(--fd-content-pad); }
.fd-auto-grid { display: grid; grid-template-columns: 60px minmax(200px, 1.5fr) 140px minmax(160px, 1fr) 300px; gap: var(--fd-space-2); align-items: center; min-height: 52px; padding: 0 var(--fd-content-pad); }
.fd-auto-grid--head { position: sticky; top: 0; z-index: 2; min-height: 32px; border-bottom: 1px solid var(--fd-border); background: var(--fd-bg); color: var(--fd-text-muted); font-size: var(--fd-text-11); }
.fd-auto-row { border-bottom: 1px solid var(--fd-border-subtle); }
.fd-auto-toggle { width: 44px; height: 22px; border: 1px solid var(--fd-border); border-radius: 12px; background: var(--fd-bg); color: var(--fd-text-muted); font-size: var(--fd-text-10); cursor: pointer; }
.fd-auto-toggle--on { border-color: var(--fd-accent); background: var(--fd-accent); color: var(--fd-accent-contrast); }
.fd-auto-name { display: flex; flex-direction: column; gap: 1px; min-width: 0; }
.fd-auto-name strong { font-size: var(--fd-text-12); font-weight: var(--fd-weight-medium); }
.fd-auto-name small, .fd-auto-muted { color: var(--fd-text-muted); font-size: var(--fd-text-11); }
.fd-auto-actions { display: flex; gap: 2px; justify-content: flex-end; }
.fd-auto-form { display: flex; flex-direction: column; gap: var(--fd-space-3); }
.fd-auto-form label { display: flex; flex-direction: column; gap: 4px; color: var(--fd-text-secondary); font-size: var(--fd-text-11); }
.fd-auto-hint { margin: 0; color: var(--fd-text-muted); font-size: var(--fd-text-11); line-height: var(--fd-line-body); }
.fd-auto-form__actions { display: flex; justify-content: flex-end; gap: var(--fd-space-2); padding-top: var(--fd-space-2); border-top: 1px solid var(--fd-border-subtle); }
.fd-auto-event, .fd-auto-run { display: flex; gap: 8px; padding: 6px 0; border-bottom: 1px solid var(--fd-border-subtle); font-size: var(--fd-text-11); }
.fd-auto-event strong { color: var(--fd-accent); }
</style>
