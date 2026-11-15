<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRoute, useRouter } from 'vue-router'

import CommandPalette from '@/components/CommandPalette.vue'
import ShortcutsDialog from '@/components/ShortcutsDialog.vue'
import AppHeader from '@/components/shell/AppHeader.vue'
import AppSidebar from '@/components/shell/AppSidebar.vue'
import StatusStrip from '@/components/shell/StatusStrip.vue'
import FdKbd from '@/components/ui/FdKbd.vue'
import { useGlobalShortcuts } from '@/composables/useShortcuts'
import { normaliseRoute } from '@/lib/routes'
import { useAppStore } from '@/stores/app'
import { useUiStore } from '@/stores/ui'

const app = useAppStore()
const ui = useUiStore()
const route = useRoute()
const router = useRouter()
const { t } = useI18n()

const main = ref<HTMLElement | null>(null)
let stopViewportTracking: (() => void) | null = null

const collapsed = computed(
  () => app.sidebarCollapsed || ui.isNarrowViewport,
)
const canToggle = computed(() => !ui.isNarrowViewport)

function toggleSidebar(): void {
  if (!canToggle.value) {
    return
  }
  void app.setSidebarCollapsed(!app.sidebarCollapsed)
}

function focusMain(event: Event): void {
  event.preventDefault()
  main.value?.focus()
}

onMounted(() => {
  stopViewportTracking = ui.startViewportTracking()
  const restored = normaliseRoute(app.settings?.lastRoute ?? '/my-issues')
  if (
    restored !== route.path &&
    (route.path === '/' || route.path === '/my-issues')
  ) {
    void router.replace(restored)
  }
})

onBeforeUnmount(() => {
  stopViewportTracking?.()
})

watch(
  () => route.path,
  (path) => {
    app.rememberRoute(path)
  },
  { immediate: true },
)

useGlobalShortcuts()
</script>

<template>
  <a class="fd-skip-link" href="#fd-main" @click="focusMain">
    {{ t('common.skipToContent') }}
  </a>

  <div class="fd-shell" data-testid="app-shell">
    <div class="fd-shell__body">
      <AppSidebar
        :collapsed="collapsed"
        :can-toggle="canToggle"
        @toggle="toggleSidebar"
      />
      <div class="fd-shell__main">
        <AppHeader />
        <main
          id="fd-main"
          ref="main"
          class="fd-shell__content"
          data-testid="main-content"
          tabindex="-1"
        >
          <RouterView />
        </main>
      </div>
    </div>
    <StatusStrip />
  </div>

  <CommandPalette />
  <ShortcutsDialog />

  <Transition name="fd-chord">
    <div v-if="ui.chordPending" class="fd-chord-hint" role="status">
      <FdKbd :keys="['G']" />
      <span>{{ t('shortcuts.chordHint') }}</span>
    </div>
  </Transition>
</template>

<style scoped>
.fd-shell {
  display: grid;
  grid-template-columns: minmax(0, 1fr);
  grid-template-rows: minmax(0, 1fr) var(--fd-status-height);
  width: 100%;
  height: 100vh;
  overflow: hidden;
  background: var(--fd-bg);
}

.fd-shell__body {
  display: flex;
  width: 100%;
  min-width: 0;
  min-height: 0;
  overflow: hidden;
}

.fd-shell__main {
  display: flex;
  flex: 1 1 0;
  flex-direction: column;
  width: 0;
  min-width: 0;
  min-height: 0;
}

.fd-shell__content {
  flex: 1 1 0;
  width: 100%;
  min-width: 0;
  min-height: 0;
  overflow-x: hidden;
  overflow-y: auto;
  outline: none;
}

.fd-skip-link {
  position: fixed;
  top: 8px;
  left: 8px;
  z-index: var(--fd-z-tooltip);
  padding: 6px 10px;
  border: 1px solid var(--fd-border);
  border-radius: var(--fd-radius-control);
  background: var(--fd-surface-raised);
  color: var(--fd-text);
  font-size: var(--fd-text-12);
  box-shadow: var(--fd-shadow-popover);
  transform: translateY(-160%);
  transition: transform var(--fd-motion-hover) var(--fd-ease-standard);
}

.fd-skip-link:focus {
  transform: translateY(0);
}

.fd-chord-hint {
  position: fixed;
  bottom: calc(var(--fd-status-height) + var(--fd-space-2));
  left: var(--fd-space-4);
  z-index: var(--fd-z-dropdown);
  display: flex;
  align-items: center;
  gap: var(--fd-space-2);
  padding: 6px 10px;
  border: 1px solid var(--fd-border);
  border-radius: var(--fd-radius-panel);
  background: var(--fd-surface-raised);
  color: var(--fd-text-secondary);
  font-size: var(--fd-text-12);
  box-shadow: var(--fd-shadow-popover);
}

.fd-chord-enter-active,
.fd-chord-leave-active {
  transition: opacity var(--fd-motion-hover) var(--fd-ease-standard);
}

.fd-chord-enter-from,
.fd-chord-leave-to {
  opacity: 0;
}
</style>
