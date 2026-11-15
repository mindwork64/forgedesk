import { defineStore } from 'pinia'
import { computed, ref } from 'vue'

import type { NavGroupId } from '@/navigation'

const GROUP_STATE_KEY = 'fd.sidebar.groups'

type GroupState = Record<NavGroupId, boolean>

const DEFAULT_GROUPS: GroupState = {
  workspace: true,
  planning: true,
  development: true,
  insights: true,
  system: true,
}

function loadGroups(): GroupState {
  try {
    const raw = localStorage.getItem(GROUP_STATE_KEY)
    if (!raw) {
      return { ...DEFAULT_GROUPS }
    }
    const parsed = JSON.parse(raw) as Partial<GroupState>
    return {
      workspace: parsed.workspace !== false,
      planning: parsed.planning !== false,
      development: parsed.development !== false,
      insights: parsed.insights !== false,
      system: parsed.system !== false,
    }
  } catch {
    return { ...DEFAULT_GROUPS }
  }
}

function persistGroups(groups: GroupState): void {
  try {
    localStorage.setItem(GROUP_STATE_KEY, JSON.stringify(groups))
  } catch {
    // Group collapse state is a convenience; failing to persist is non-fatal.
  }
}

export const useUiStore = defineStore('ui', () => {
  const commandPaletteOpen = ref(false)
  const shortcutsOpen = ref(false)
  const groups = ref<GroupState>(loadGroups())
  const viewportWidth = ref(
    typeof window === 'undefined' ? 1440 : window.innerWidth,
  )
  const chordPending = ref(false)

  const isNarrowViewport = computed(() => viewportWidth.value < 1200)

  let chordTimer: ReturnType<typeof setTimeout> | null = null

  function openCommandPalette(): void {
    shortcutsOpen.value = false
    commandPaletteOpen.value = true
  }

  function closeCommandPalette(): void {
    commandPaletteOpen.value = false
  }

  function toggleCommandPalette(): void {
    if (commandPaletteOpen.value) {
      closeCommandPalette()
    } else {
      openCommandPalette()
    }
  }

  function openShortcuts(): void {
    commandPaletteOpen.value = false
    shortcutsOpen.value = true
  }

  function closeShortcuts(): void {
    shortcutsOpen.value = false
  }

  function toggleGroup(id: NavGroupId): void {
    groups.value = { ...groups.value, [id]: !groups.value[id] }
    persistGroups(groups.value)
  }

  function showChordHint(): void {
    chordPending.value = true
    if (chordTimer) {
      clearTimeout(chordTimer)
    }
    chordTimer = setTimeout(() => {
      chordPending.value = false
      chordTimer = null
    }, 1200)
  }

  function hideChordHint(): void {
    chordPending.value = false
    if (chordTimer) {
      clearTimeout(chordTimer)
      chordTimer = null
    }
  }

  function updateViewportWidth(): void {
    viewportWidth.value = window.innerWidth
  }

  function startViewportTracking(): () => void {
    updateViewportWidth()
    window.addEventListener('resize', updateViewportWidth)
    return () => window.removeEventListener('resize', updateViewportWidth)
  }

  return {
    commandPaletteOpen,
    shortcutsOpen,
    groups,
    viewportWidth,
    isNarrowViewport,
    chordPending,
    openCommandPalette,
    closeCommandPalette,
    toggleCommandPalette,
    openShortcuts,
    closeShortcuts,
    toggleGroup,
    showChordHint,
    hideChordHint,
    startViewportTracking,
  }
})
