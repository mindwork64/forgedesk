import { onBeforeUnmount, onMounted } from 'vue'
import { useRouter } from 'vue-router'

import { useAppStore } from '@/stores/app'
import { useUiStore } from '@/stores/ui'

const CHORD_TIMEOUT_MS = 1200

const CHORD_ROUTES: Record<string, string> = {
  i: '/inbox',
  m: '/my-issues',
  p: '/projects',
  r: '/repositories',
}

function isTextEntry(target: EventTarget | null): boolean {
  const element = target as HTMLElement | null
  if (!element) {
    return false
  }
  return (
    element.tagName === 'INPUT' ||
    element.tagName === 'TEXTAREA' ||
    element.tagName === 'SELECT' ||
    element.isContentEditable
  )
}

function dialogIsOpen(): boolean {
  return document.querySelector('[role="dialog"][aria-modal="true"]') !== null
}

export function useGlobalShortcuts(): void {
  const router = useRouter()
  const ui = useUiStore()
  const app = useAppStore()

  let chordPending = false
  let chordTimer: ReturnType<typeof setTimeout> | null = null

  function resetChord(): void {
    chordPending = false
    if (chordTimer) {
      clearTimeout(chordTimer)
      chordTimer = null
    }
    ui.hideChordHint()
  }

  function onKeydown(event: KeyboardEvent): void {
    const modifier = event.ctrlKey || event.metaKey

    if (modifier && event.key.toLowerCase() === 'k') {
      if (app.needsOnboarding) {
        return
      }
      const otherDialog =
        dialogIsOpen() &&
        !ui.commandPaletteOpen &&
        !ui.shortcutsOpen
      if (otherDialog) {
        return
      }
      event.preventDefault()
      ui.toggleCommandPalette()
      return
    }

    if (event.key === 'Escape' && chordPending) {
      resetChord()
      return
    }

    if (app.needsOnboarding || dialogIsOpen() || isTextEntry(event.target)) {
      return
    }

    if (event.key === '?') {
      event.preventDefault()
      ui.openShortcuts()
      return
    }

    if (chordPending) {
      chordPending = false
      if (chordTimer) {
        clearTimeout(chordTimer)
        chordTimer = null
      }
      ui.hideChordHint()
      const path = CHORD_ROUTES[event.key.toLowerCase()]
      if (path) {
        event.preventDefault()
        void router.push(path)
      }
      return
    }

    if (
      event.key.toLowerCase() === 'g' &&
      !event.altKey &&
      !event.shiftKey &&
      !event.ctrlKey &&
      !event.metaKey
    ) {
      chordPending = true
      ui.showChordHint()
      chordTimer = setTimeout(resetChord, CHORD_TIMEOUT_MS)
    }
  }

  onMounted(() => {
    window.addEventListener('keydown', onKeydown)
  })

  onBeforeUnmount(() => {
    window.removeEventListener('keydown', onKeydown)
    if (chordTimer) {
      clearTimeout(chordTimer)
    }
  })
}
