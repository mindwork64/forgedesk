import { defineStore } from 'pinia'
import { ref } from 'vue'

import { api, toIpcError } from '@/lib/ipc'
import type { InsightsSummary } from '@/types/ipc'

export const useInsightsStore = defineStore('insights', () => {
  const summary = ref<InsightsSummary | null>(null)
  const rangeDays = ref(30)
  const loading = ref(false)
  const error = ref<ReturnType<typeof toIpcError> | null>(null)

  async function load(): Promise<void> {
    loading.value = true
    error.value = null
    try {
      summary.value = await api.getInsights(rangeDays.value)
    } catch (caught) {
      error.value = toIpcError(caught)
    } finally {
      loading.value = false
    }
  }

  async function setRange(days: number): Promise<void> {
    rangeDays.value = days
    await load()
  }

  return { summary, rangeDays, loading, error, load, setRange }
})
