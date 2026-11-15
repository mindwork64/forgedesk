import { defineStore } from 'pinia'
import { ref } from 'vue'

import { i18n } from '@/i18n'
import { api, toIpcError } from '@/lib/ipc'
import { useToastStore } from '@/stores/toasts'
import type { Dashboard } from '@/types/ipc'

export const useDashboardStore = defineStore('dashboard', () => {
  const t = i18n.global.t
  const toast = useToastStore()
  const dashboard = ref<Dashboard | null>(null)
  const loading = ref(false)
  const error = ref<ReturnType<typeof toIpcError> | null>(null)

  async function load(): Promise<void> {
    loading.value = true
    error.value = null
    try {
      dashboard.value = await api.dashboardGet()
    } catch (caught) {
      error.value = toIpcError(caught)
    } finally {
      loading.value = false
    }
  }

  async function apply(action: Promise<Dashboard>): Promise<void> {
    try {
      dashboard.value = await action
    } catch (caught) {
      toast.error(t('dashboardPage.updateError'), toIpcError(caught).message)
    }
  }

  async function add(kind: string, width: number): Promise<void> {
    await apply(api.dashboardAddWidget(kind, width))
  }

  async function remove(widgetId: string): Promise<void> {
    await apply(api.dashboardRemoveWidget(widgetId))
  }

  async function resize(widgetId: string, width: number): Promise<void> {
    await apply(api.dashboardUpdateWidth(widgetId, width))
  }

  async function move(widgetId: string, direction: -1 | 1): Promise<void> {
    const current = dashboard.value?.widgets ?? []
    const index = current.findIndex((widget) => widget.id === widgetId)
    const next = index + direction
    if (index < 0 || next < 0 || next >= current.length) return
    const ids = current.map((widget) => widget.id)
    ;[ids[index], ids[next]] = [ids[next]!, ids[index]!]
    await apply(api.dashboardReorder(ids))
  }

  return { dashboard, loading, error, load, add, remove, resize, move }
})
