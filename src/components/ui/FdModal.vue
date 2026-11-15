<script setup lang="ts">
import { X } from '@lucide/vue'
import { nextTick, onBeforeUnmount, ref, useId, watch } from 'vue'
import { useI18n } from 'vue-i18n'

import FdIconButton from './FdIconButton.vue'

const props = withDefaults(
  defineProps<{
    title: string
    size?: 'sm' | 'md' | 'lg'
    persistent?: boolean
  }>(),
  { size: 'md', persistent: false },
)

const open = defineModel<boolean>({ default: false })
const { t } = useI18n()
const titleId = useId()
const panel = ref<HTMLElement | null>(null)
let previousFocus: HTMLElement | null = null

const FOCUSABLE =
  'a[href], button:not([disabled]), input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])'

function close(): void {
  if (props.persistent) {
    return
  }
  open.value = false
}

function onKeydown(event: KeyboardEvent): void {
  if (event.key === 'Escape' && !props.persistent) {
    event.stopPropagation()
    close()
    return
  }
  if (event.key !== 'Tab' || !panel.value) {
    return
  }
  const nodes = Array.from(
    panel.value.querySelectorAll<HTMLElement>(FOCUSABLE),
  ).filter((node) => node.offsetParent !== null || node === document.activeElement)
  if (nodes.length === 0) {
    event.preventDefault()
    panel.value.focus()
    return
  }
  const first = nodes[0]
  const last = nodes[nodes.length - 1]
  if (!first || !last) {
    return
  }
  if (document.activeElement === panel.value) {
    event.preventDefault()
    ;(event.shiftKey ? last : first).focus()
    return
  }
  if (event.shiftKey && document.activeElement === first) {
    event.preventDefault()
    last.focus()
  } else if (!event.shiftKey && document.activeElement === last) {
    event.preventDefault()
    first.focus()
  }
}

watch(open, async (value) => {
  if (value) {
    previousFocus = document.activeElement as HTMLElement | null
    document.body.style.overflow = 'hidden'
    await nextTick()
    // Focus the dialog container itself. Focusing the first control (often the
    // close button) would immediately show its tooltip over the dialog body.
    panel.value?.focus()
  } else {
    document.body.style.overflow = ''
    previousFocus?.focus()
    previousFocus = null
  }
})

onBeforeUnmount(() => {
  document.body.style.overflow = ''
})
</script>

<template>
  <Teleport to="body">
    <Transition name="fd-modal">
      <div
        v-if="open"
        class="fd-modal-backdrop"
        @mousedown.self="close"
      >
        <div
          ref="panel"
          class="fd-modal-panel"
          :class="`fd-modal-panel--${size}`"
          role="dialog"
          aria-modal="true"
          :aria-labelledby="titleId"
          tabindex="-1"
          @keydown="onKeydown"
        >
          <header class="fd-modal__header">
            <h2 :id="titleId" class="fd-modal__title">{{ title }}</h2>
            <FdIconButton
              v-if="!persistent"
              :label="t('common.close')"
              @click="close"
            >
              <X :size="16" />
            </FdIconButton>
          </header>
          <div class="fd-modal__body">
            <slot />
          </div>
          <footer v-if="$slots.footer" class="fd-modal__footer">
            <slot name="footer" />
          </footer>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.fd-modal-backdrop {
  position: fixed;
  inset: 0;
  z-index: var(--fd-z-modal);
  display: flex;
  align-items: flex-start;
  justify-content: center;
  padding: 8vh var(--fd-space-6) var(--fd-space-6);
  background: var(--fd-overlay);
  overflow-y: auto;
}

.fd-modal-panel {
  display: flex;
  flex-direction: column;
  width: 100%;
  max-height: calc(100vh - 12vh);
  border: 1px solid var(--fd-border);
  border-radius: var(--fd-radius-modal);
  background: var(--fd-surface);
  box-shadow: var(--fd-shadow-modal);
  outline: none;
}

.fd-modal-panel--sm {
  max-width: 420px;
}

.fd-modal-panel--md {
  max-width: 560px;
}

.fd-modal-panel--lg {
  max-width: 720px;
}

.fd-modal__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--fd-space-3);
  padding: 14px var(--fd-space-5) 10px;
}

.fd-modal__title {
  font-size: var(--fd-text-16);
  font-weight: var(--fd-weight-semibold);
  line-height: var(--fd-line-tight);
}

.fd-modal__body {
  padding: 6px var(--fd-space-5) var(--fd-space-5);
  overflow-y: auto;
}

.fd-modal__footer {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: var(--fd-space-2);
  padding: 12px var(--fd-space-5);
  border-top: 1px solid var(--fd-border-subtle);
}

.fd-modal-enter-active,
.fd-modal-leave-active {
  transition: opacity var(--fd-motion-drawer) var(--fd-ease-standard);
}

.fd-modal-enter-active .fd-modal-panel,
.fd-modal-leave-active .fd-modal-panel {
  transition: transform var(--fd-motion-drawer) var(--fd-ease-standard);
}

.fd-modal-enter-from,
.fd-modal-leave-to {
  opacity: 0;
}

.fd-modal-enter-from .fd-modal-panel,
.fd-modal-leave-to .fd-modal-panel {
  transform: translateY(6px);
}
</style>
