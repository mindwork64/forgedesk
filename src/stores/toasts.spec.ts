import { createPinia, setActivePinia } from 'pinia'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'

import { useToastStore } from './toasts'

describe('toast store', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.useFakeTimers()
  })

  afterEach(() => {
    vi.useRealTimers()
  })

  it('auto-dismisses success toasts', () => {
    const toast = useToastStore()
    toast.success('Saved')
    expect(toast.items).toHaveLength(1)
    vi.advanceTimersByTime(4_000)
    expect(toast.items).toHaveLength(0)
  })

  it('keeps error toasts longer and allows manual dismissal', () => {
    const toast = useToastStore()
    const id = toast.error('Failed', 'Details')
    vi.advanceTimersByTime(4_100)
    expect(toast.items).toHaveLength(1)
    toast.dismiss(id)
    expect(toast.items).toHaveLength(0)
  })

  it('caps the stack at four visible toasts', () => {
    const toast = useToastStore()
    for (let index = 0; index < 6; index += 1) {
      toast.info(`Item ${index}`)
    }
    expect(toast.items).toHaveLength(4)
    expect(toast.items[0]?.title).toBe('Item 2')
  })
})
