import { defineStore } from 'pinia'
import { ref } from 'vue'

import { i18n } from '@/i18n'
import { api, toIpcError } from '@/lib/ipc'
import { useToastStore } from '@/stores/toasts'
import type { AutomationEvent, AutomationInput, AutomationRule, AutomationRun } from '@/types/ipc'

export const useAutomationStore = defineStore('automations', () => {
  const t = i18n.global.t
  const toast = useToastStore()
  const rules = ref<AutomationRule[]>([])
  const preview = ref<AutomationEvent[]>([])
  const history = ref<AutomationRun[]>([])
  const loading = ref(false)
  const busy = ref(false)
  const error = ref<ReturnType<typeof toIpcError> | null>(null)

  async function load(): Promise<void> {
    loading.value = true
    error.value = null
    try {
      rules.value = await api.automationListRules()
    } catch (caught) {
      error.value = toIpcError(caught)
    } finally {
      loading.value = false
    }
  }

  async function create(input: AutomationInput): Promise<boolean> {
    busy.value = true
    try {
      const rule = await api.automationCreate(input)
      rules.value = [...rules.value, rule]
      toast.success(t('automations.created'), rule.name)
      return true
    } catch (caught) {
      toast.error(t('automations.createError'), toIpcError(caught).message)
      return false
    } finally {
      busy.value = false
    }
  }

  async function toggle(rule: AutomationRule): Promise<void> {
    try {
      const updated = await api.automationToggle(rule.id, !rule.enabled)
      rules.value = rules.value.map((item) => (item.id === updated.id ? updated : item))
    } catch (caught) {
      toast.error(t('automations.toggleError'), toIpcError(caught).message)
    }
  }

  async function remove(rule: AutomationRule): Promise<void> {
    try {
      await api.automationDelete(rule.id)
      rules.value = rules.value.filter((item) => item.id !== rule.id)
      toast.success(t('automations.deleted'))
    } catch (caught) {
      toast.error(t('automations.deleteError'), toIpcError(caught).message)
    }
  }

  async function loadPreview(rule: AutomationRule): Promise<void> {
    preview.value = await api.automationPreview(rule.id)
  }

  async function loadHistory(rule: AutomationRule): Promise<void> {
    history.value = await api.automationRuns(rule.id)
  }

  async function run(rule: AutomationRule): Promise<void> {
    try {
      const count = await api.automationRun(rule.id)
      toast.success(t('automations.ran'), String(count))
      await load()
    } catch (caught) {
      toast.error(t('automations.runError'), toIpcError(caught).message)
    }
  }

  return { rules, preview, history, loading, busy, error, load, create, toggle, remove, loadPreview, loadHistory, run }
})
