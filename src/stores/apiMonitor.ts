import { defineStore } from 'pinia'
import { ref } from 'vue'

import { i18n } from '@/i18n'
import { copyText } from '@/lib/clipboard'
import { api, toIpcError } from '@/lib/ipc'
import { useToastStore } from '@/stores/toasts'
import type { ApiMonitorState } from '@/types/ipc'

export const useApiMonitorStore = defineStore('apiMonitor', () => {
  const t = i18n.global.t
  const toast = useToastStore()
  const state = ref<ApiMonitorState | null>(null)
  const loading = ref(false)
  const error = ref<ReturnType<typeof toIpcError> | null>(null)

  async function load(): Promise<void> {
    loading.value = true
    error.value = null
    try {
      state.value = await api.getApiMonitor()
    } catch (caught) {
      error.value = toIpcError(caught)
    } finally {
      loading.value = false
    }
  }

  async function exportDiagnostics(): Promise<void> {
    try {
      const diagnostics = await api.exportApiDiagnostics()
      const copied = await copyText(diagnostics)
      if (copied) {
        toast.success(t('apiMonitor.diagnosticsCopied'))
      } else {
        toast.error(t('apiMonitor.diagnosticsError'))
      }
    } catch (caught) {
      const wire = toIpcError(caught)
      toast.error(t('apiMonitor.diagnosticsError'), wire.message, wire.detail)
    }
  }

  return { state, loading, error, load, exportDiagnostics }
})
