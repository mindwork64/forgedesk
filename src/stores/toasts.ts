import { defineStore } from 'pinia'
import { ref } from 'vue'

export type ToastKind = 'success' | 'error' | 'warning' | 'info'

export interface ToastInput {
  kind: ToastKind
  title: string
  message?: string
  detail?: string
  duration?: number
}

export interface Toast extends ToastInput {
  id: number
}

let nextToastId = 0

const DEFAULT_DURATION: Record<ToastKind, number> = {
  success: 4000,
  info: 4000,
  warning: 6000,
  error: 8000,
}

export const useToastStore = defineStore('toasts', () => {
  const items = ref<Toast[]>([])
  const timers = new Map<number, ReturnType<typeof setTimeout>>()

  function dismiss(id: number): void {
    const timer = timers.get(id)
    if (timer) {
      clearTimeout(timer)
      timers.delete(id)
    }
    items.value = items.value.filter((item) => item.id !== id)
  }

  function push(input: ToastInput): number {
    const id = ++nextToastId
    const duration = input.duration ?? DEFAULT_DURATION[input.kind]
    items.value = [...items.value.slice(-3), { ...input, id }]
    if (duration > 0) {
      timers.set(
        id,
        setTimeout(() => dismiss(id), duration),
      )
    }
    return id
  }

  function success(title: string, message?: string): number {
    return push({ kind: 'success', title, message })
  }

  function error(title: string, message?: string, detail?: string): number {
    return push({ kind: 'error', title, message, detail })
  }

  function warning(title: string, message?: string): number {
    return push({ kind: 'warning', title, message })
  }

  function info(title: string, message?: string): number {
    return push({ kind: 'info', title, message })
  }

  return { items, push, dismiss, success, error, warning, info }
})
