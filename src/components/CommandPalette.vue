<script setup lang="ts">
import {
  CircleDot,
  HardDrive,
  Keyboard,
  PanelLeftClose,
  Search,
  Settings,
  Sun,
} from '@lucide/vue'
import type { Component } from 'vue'
import { computed, nextTick, onBeforeUnmount, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRoute, useRouter } from 'vue-router'

import FdKbd from '@/components/ui/FdKbd.vue'
import { api } from '@/lib/ipc'
import { statusLabel } from '@/lib/issue'
import { NAV_GROUPS, UTILITY_NAV } from '@/navigation'
import { useAppStore } from '@/stores/app'
import { useIssueStore } from '@/stores/issues'
import { useUiStore } from '@/stores/ui'
import type { IssueSummary } from '@/types/ipc'

interface PaletteCommand {
  id: string
  label: string
  hint?: string
  icon: Component
  shortcut?: string[]
  run: () => void
}

interface PaletteSection {
  key: 'actions' | 'issues' | 'navigation'
  label: string
  offset: number
  items: PaletteCommand[]
}

const ui = useUiStore()
const app = useAppStore()
const issueStore = useIssueStore()
const route = useRoute()
const router = useRouter()
const { t, te } = useI18n()

const query = ref('')
const selectedIndex = ref(0)
const input = ref<HTMLInputElement | null>(null)
const issueResults = ref<IssueSummary[]>([])
const issueSearching = ref(false)
let previousFocus: HTMLElement | null = null
let issueSearchTimer: ReturnType<typeof setTimeout> | null = null
let issueSearchId = 0

function close(): void {
  ui.closeCommandPalette()
}

function run(command: PaletteCommand): void {
  close()
  command.run()
}

const actionCommands = computed<PaletteCommand[]>(() => [
  {
    id: 'theme',
    label: t('palette.toggleTheme'),
    icon: Sun,
    run: () => {
      void app.setTheme(app.resolvedTheme === 'dark' ? 'light' : 'dark')
    },
  },
  {
    id: 'sidebar',
    label: t('palette.toggleSidebar'),
    icon: PanelLeftClose,
    run: () => {
      void app.setSidebarCollapsed(!app.sidebarCollapsed)
    },
  },
  {
    id: 'backup',
    label: t('palette.createBackup'),
    icon: HardDrive,
    run: () => {
      void app.createBackup()
    },
  },
  {
    id: 'settings',
    label: t('palette.openSettings'),
    icon: Settings,
    run: () => {
      void router.push('/settings/general')
    },
  },
  {
    id: 'shortcuts',
    label: t('palette.showShortcuts'),
    icon: Keyboard,
    run: () => {
      ui.openShortcuts()
    },
  },
])

const navigationCommands = computed<PaletteCommand[]>(() => {
  const items = [...NAV_GROUPS.flatMap((group) => group.items), ...UTILITY_NAV]
  return items.map((item) => ({
    id: `nav-${item.key}`,
    label: t(item.labelKey),
    hint: item.path,
    icon: item.icon,
    run: () => {
      void router.push(item.path)
    },
  }))
})

function matches(command: PaletteCommand, needle: string): boolean {
  if (!needle) {
    return true
  }
  return (
    command.label.toLowerCase().includes(needle) ||
    (command.hint?.toLowerCase().includes(needle) ?? false)
  )
}

const needle = computed(() => query.value.trim().toLowerCase())

const issueCommands = computed<PaletteCommand[]>(() =>
  issueResults.value.map((issue) => ({
    id: `issue-${issue.id}`,
    label: `${issue.key} · ${issue.title}`,
    hint: statusLabel(issue.status, t, te),
    icon: CircleDot,
    run: () => {
      const alreadyOpen =
        route.path === '/my-issues' && route.query.issue === issue.id
      if (alreadyOpen) {
        void issueStore.selectIssue(issue.id)
      } else {
        void router.push({ path: '/my-issues', query: { issue: issue.id } })
      }
    },
  })),
)

const sections = computed<PaletteSection[]>(() => {
  const actions = actionCommands.value.filter((item) => matches(item, needle.value))
  const navigation = navigationCommands.value.filter((item) =>
    matches(item, needle.value),
  )
  const issues = issueCommands.value
  let offset = 0
  const result: PaletteSection[] = []
  if (actions.length) {
    result.push({
      key: 'actions',
      label: t('palette.actions'),
      offset,
      items: actions,
    })
    offset += actions.length
  }
  if (issues.length) {
    result.push({
      key: 'issues',
      label: t('palette.issues'),
      offset,
      items: issues,
    })
    offset += issues.length
  }
  if (navigation.length) {
    result.push({
      key: 'navigation',
      label: t('palette.navigation'),
      offset,
      items: navigation,
    })
  }
  return result
})

const flatItems = computed(() => sections.value.flatMap((section) => section.items))

watch(needle, (value) => {
  selectedIndex.value = 0
  if (issueSearchTimer) {
    clearTimeout(issueSearchTimer)
    issueSearchTimer = null
  }
  if (!ui.commandPaletteOpen || value.length < 2) {
    issueResults.value = []
    issueSearching.value = false
    issueSearchId += 1
    return
  }
  const requestId = ++issueSearchId
  issueResults.value = []
  issueSearching.value = true
  issueSearchTimer = setTimeout(async () => {
    try {
      const result = await api.listIssues({
        search: value,
        sort: [{ field: 'updatedAt', direction: 'desc' }],
        includeArchived: false,
        limit: 6,
      })
      if (requestId === issueSearchId) {
        issueResults.value = result
      }
    } catch {
      if (requestId === issueSearchId) {
        issueResults.value = []
      }
    } finally {
      if (requestId === issueSearchId) {
        issueSearching.value = false
      }
    }
  }, 180)
})

watch(
  () => ui.commandPaletteOpen,
  async (open) => {
    if (open) {
      query.value = ''
      selectedIndex.value = 0
      previousFocus = document.activeElement as HTMLElement | null
      document.body.style.overflow = 'hidden'
      await nextTick()
      input.value?.focus()
    } else {
      document.body.style.overflow = ''
      previousFocus?.focus()
      previousFocus = null
      issueSearchId += 1
      issueResults.value = []
      issueSearching.value = false
      if (issueSearchTimer) {
        clearTimeout(issueSearchTimer)
        issueSearchTimer = null
      }
    }
  },
)

watch(selectedIndex, async () => {
  await nextTick()
  document
    .getElementById(`fd-palette-item-${selectedIndex.value}`)
    ?.scrollIntoView({ block: 'nearest' })
})

function onKeydown(event: KeyboardEvent): void {
  if (event.key === 'Escape') {
    event.preventDefault()
    close()
    return
  }
  if (event.key === 'Tab') {
    // The palette is a combobox; focus stays in the input.
    event.preventDefault()
    return
  }
  if (event.key === 'ArrowDown') {
    event.preventDefault()
    const count = flatItems.value.length
    if (count > 0) {
      selectedIndex.value = (selectedIndex.value + 1) % count
    }
    return
  }
  if (event.key === 'ArrowUp') {
    event.preventDefault()
    const count = flatItems.value.length
    if (count > 0) {
      selectedIndex.value = (selectedIndex.value - 1 + count) % count
    }
    return
  }
  if (event.key === 'Enter') {
    event.preventDefault()
    const command = flatItems.value[selectedIndex.value]
    if (command) {
      run(command)
    }
  }
}


onBeforeUnmount(() => {
  document.body.style.overflow = ''
  if (issueSearchTimer) {
    clearTimeout(issueSearchTimer)
  }
})
</script>

<template>
  <Teleport to="body">
    <Transition name="fd-palette">
      <div
        v-if="ui.commandPaletteOpen"
        class="fd-palette-backdrop"
        @mousedown.self="close"
      >
        <div
          ref="panel"
          class="fd-palette"
          role="dialog"
          aria-modal="true"
          :aria-label="t('palette.placeholder')"
          @keydown="onKeydown"
        >
          <div class="fd-palette__search">
            <Search :size="16" class="fd-palette__search-icon" aria-hidden="true" />
            <input
              ref="input"
              v-model="query"
              class="fd-palette__input"
              type="text"
              role="combobox"
              aria-autocomplete="list"
              aria-expanded="true"
              :aria-controls="'fd-palette-list'"
              :aria-activedescendant="
                flatItems.length > 0 ? `fd-palette-item-${selectedIndex}` : undefined
              "
              :placeholder="t('palette.placeholder')"
              autocomplete="off"
              spellcheck="false"
            />
            <FdKbd :keys="['Esc']" />
          </div>

          <div v-if="flatItems.length > 0" id="fd-palette-list" class="fd-palette__results" role="listbox">
            <section
              v-for="section in sections"
              :key="section.key"
              class="fd-palette__group"
            >
              <p class="fd-palette__group-label">{{ section.label }}</p>
              <div
                v-for="(item, itemIndex) in section.items"
                :id="`fd-palette-item-${section.offset + itemIndex}`"
                :key="item.id"
                class="fd-palette__item"
                :class="{
                  'fd-palette__item--selected':
                    selectedIndex === section.offset + itemIndex,
                }"
                role="option"
                :aria-selected="selectedIndex === section.offset + itemIndex"
                @mouseenter="selectedIndex = section.offset + itemIndex"
                @mousedown.prevent
                @click="run(item)"
              >
                <component
                  :is="item.icon"
                  :size="16"
                  class="fd-palette__item-icon"
                  aria-hidden="true"
                />
                <span class="fd-palette__item-label">{{ item.label }}</span>
                <span v-if="item.hint" class="fd-palette__item-hint">
                  {{ item.hint }}
                </span>
                <FdKbd v-if="item.shortcut" :keys="item.shortcut" />
              </div>
            </section>
          </div>

          <div v-else class="fd-palette__empty" role="status">
            <p class="fd-palette__empty-title">
              {{ issueSearching ? t('palette.searching') : t('palette.noResults') }}
            </p>
            <p class="fd-palette__empty-hint">
              {{
                issueSearching
                  ? t('palette.searchingHint')
                  : t('palette.noResultsHint')
              }}
            </p>
          </div>

          <footer class="fd-palette__footer">
            <div class="fd-palette__hints">
              <span><FdKbd :keys="['↑↓']" /> {{ t('palette.navigation') }}</span>
              <span><FdKbd :keys="['↵']" /> {{ t('common.open') }}</span>
              <span><FdKbd :keys="['Esc']" /> {{ t('common.close') }}</span>
            </div>
            <span class="fd-palette__note">{{ t('palette.entitySearchNote') }}</span>
          </footer>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.fd-palette-backdrop {
  position: fixed;
  inset: 0;
  z-index: var(--fd-z-modal);
  display: flex;
  align-items: flex-start;
  justify-content: center;
  padding: 30vh var(--fd-space-4) var(--fd-space-4);
  background: var(--fd-overlay);
}

.fd-palette {
  display: flex;
  flex-direction: column;
  width: min(640px, calc(100vw - 32px));
  max-height: 520px;
  overflow: hidden;
  border: 1px solid var(--fd-border);
  border-radius: var(--fd-radius-modal);
  background: var(--fd-surface-raised);
  box-shadow: var(--fd-shadow-modal);
}

.fd-palette__search {
  display: flex;
  flex: none;
  align-items: center;
  gap: var(--fd-space-2);
  height: 46px;
  padding: 0 var(--fd-space-4);
  border-bottom: 1px solid var(--fd-border-subtle);
}

.fd-palette__search-icon {
  flex: none;
  color: var(--fd-text-muted);
}

.fd-palette__input {
  flex: 1;
  min-width: 0;
  height: 100%;
  border: 0;
  background: transparent;
  color: var(--fd-text);
  font-size: var(--fd-text-14);
}

.fd-palette__input:focus {
  outline: none;
}

.fd-palette__input::placeholder {
  color: var(--fd-text-muted);
}

.fd-palette__results {
  flex: 1 1 auto;
  min-height: 0;
  padding: var(--fd-space-2);
  overflow-y: auto;
}

.fd-palette__group + .fd-palette__group {
  margin-top: var(--fd-space-2);
}

.fd-palette__group-label {
  padding: 4px 8px;
  color: var(--fd-text-muted);
  font-size: var(--fd-text-11);
  font-weight: var(--fd-weight-semibold);
  letter-spacing: 0.05em;
  text-transform: uppercase;
}

.fd-palette__item {
  display: flex;
  align-items: center;
  gap: var(--fd-space-2);
  height: 36px;
  padding: 0 8px;
  border-radius: var(--fd-radius-control);
  color: var(--fd-text-secondary);
  cursor: pointer;
  user-select: none;
}

.fd-palette__item--selected {
  background: var(--fd-surface-selected);
  color: var(--fd-text);
}

.fd-palette__item-icon {
  flex: none;
  color: var(--fd-text-muted);
}

.fd-palette__item--selected .fd-palette__item-icon {
  color: var(--fd-accent);
}

.fd-palette__item-label {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  font-size: var(--fd-text-13);
  text-overflow: ellipsis;
  white-space: nowrap;
}

.fd-palette__item-hint {
  overflow: hidden;
  max-width: 200px;
  color: var(--fd-text-muted);
  font-family: var(--fd-font-mono);
  font-size: var(--fd-text-11);
  text-overflow: ellipsis;
  white-space: nowrap;
}

.fd-palette__empty {
  padding: var(--fd-space-6);
  text-align: center;
}

.fd-palette__empty-title {
  font-size: var(--fd-text-13);
  font-weight: var(--fd-weight-medium);
}

.fd-palette__empty-hint {
  margin-top: 4px;
  color: var(--fd-text-muted);
  font-size: var(--fd-text-12);
}

.fd-palette__footer {
  display: flex;
  flex: none;
  align-items: center;
  justify-content: space-between;
  gap: var(--fd-space-3);
  padding: 8px var(--fd-space-4);
  border-top: 1px solid var(--fd-border-subtle);
}

.fd-palette__hints {
  display: flex;
  align-items: center;
  gap: var(--fd-space-3);
  color: var(--fd-text-muted);
  font-size: var(--fd-text-11);
}

.fd-palette__hints span {
  display: inline-flex;
  align-items: center;
  gap: 4px;
}

.fd-palette__note {
  overflow: hidden;
  color: var(--fd-text-muted);
  font-size: var(--fd-text-11);
  text-overflow: ellipsis;
  white-space: nowrap;
}

.fd-palette-enter-active,
.fd-palette-leave-active {
  transition: opacity var(--fd-motion-drawer) var(--fd-ease-standard);
}

.fd-palette-enter-active .fd-palette,
.fd-palette-leave-active .fd-palette {
  transition: transform var(--fd-motion-drawer) var(--fd-ease-standard);
}

.fd-palette-enter-from,
.fd-palette-leave-to {
  opacity: 0;
}

.fd-palette-enter-from .fd-palette,
.fd-palette-leave-to .fd-palette {
  transform: translateY(-4px);
}
</style>
