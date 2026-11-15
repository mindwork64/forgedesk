import { defineStore } from 'pinia'
import { computed, ref } from 'vue'

import { i18n } from '@/i18n'
import { api, toIpcError } from '@/lib/ipc'
import { useToastStore } from '@/stores/toasts'
import type { InboxEvent } from '@/types/ipc'

export const useInboxStore = defineStore('inbox', () => {
  const t = i18n.global.t
  const toast = useToastStore()

  const events = ref<InboxEvent[]>([])
  const filter = ref('all')
  const loading = ref(false)
  const error = ref<ReturnType<typeof toIpcError> | null>(null)
  const unread = computed(() => events.value.filter((event) => !event.read).length)

  async function load(): Promise<void> {
    loading.value = true
    error.value = null
    try {
      events.value = await api.inboxList(filter.value)
    } catch (caught) {
      error.value = toIpcError(caught)
    } finally {
      loading.value = false
    }
  }

  async function setFilter(next: string): Promise<void> {
    filter.value = next
    await load()
  }

  async function markRead(event: InboxEvent, read: boolean): Promise<void> {
    try {
      await api.inboxMarkRead(event.id, read)
      event.read = read
    } catch (caught) {
      toast.error(t('inbox.markError'), toIpcError(caught).message)
    }
  }

  async function archive(event: InboxEvent): Promise<void> {
    try {
      await api.inboxArchive(event.id)
      events.value = events.value.filter((item) => item.id !== event.id)
      toast.success(t('inbox.archived'))
    } catch (caught) {
      toast.error(t('inbox.archiveError'), toIpcError(caught).message)
    }
  }

  async function markAllRead(): Promise<void> {
    try {
      const count = await api.inboxMarkAllRead()
      events.value = events.value.map((event) => ({ ...event, read: true }))
      toast.success(t('inbox.allRead'), String(count))
    } catch (caught) {
      toast.error(t('inbox.markError'), toIpcError(caught).message)
    }
  }

  return { events, filter, loading, error, unread, load, setFilter, markRead, archive, markAllRead }
})
