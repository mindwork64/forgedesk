<script setup lang="ts">
import { nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'

withDefaults(
  defineProps<{
    align?: 'start' | 'end'
    placement?: 'bottom' | 'top'
    width?: number
    ariaLabel?: string
  }>(),
  {
    align: 'end',
    placement: 'bottom',
    width: 280,
    ariaLabel: undefined,
  },
)

const open = defineModel<boolean>('open', { default: false })
const root = ref<HTMLElement | null>(null)
const panel = ref<HTMLElement | null>(null)
const trigger = ref<HTMLElement | null>(null)

function onDocumentPointerDown(event: MouseEvent): void {
  if (!open.value || !root.value) {
    return
  }
  if (!root.value.contains(event.target as Node)) {
    open.value = false
  }
}

function onDocumentKeydown(event: KeyboardEvent): void {
  if (event.key === 'Escape' && open.value) {
    open.value = false
    restoreTriggerFocus()
  }
}

onMounted(() => {
  document.addEventListener('mousedown', onDocumentPointerDown)
  document.addEventListener('keydown', onDocumentKeydown)
})

onBeforeUnmount(() => {
  document.removeEventListener('mousedown', onDocumentPointerDown)
  document.removeEventListener('keydown', onDocumentKeydown)
})

watch(open, async (value) => {
  if (value) {
    await nextTick()
    // Focus the panel, not its first control: focusing a copy/close button
    // would immediately show its tooltip over the panel content.
    panel.value?.focus()
  }
})

function restoreTriggerFocus(): void {
  trigger.value?.querySelector<HTMLElement>('button, a[href], [tabindex]')?.focus()
}
</script>

<template>
  <div ref="root" class="fd-popover">
    <span ref="trigger" class="fd-popover__trigger">
      <slot name="trigger" :open="open" :toggle="() => (open = !open)" />
    </span>
    <Transition name="fd-popover">
      <div
        v-if="open"
        ref="panel"
        class="fd-popover__panel"
        :class="[
          `fd-popover__panel--${placement}`,
          `fd-popover__panel--${align}`,
        ]"
        :style="{ width: `${width}px` }"
        tabindex="-1"
        role="dialog"
        :aria-label="ariaLabel"
      >
        <slot :close="() => (open = false)" />
      </div>
    </Transition>
  </div>
</template>

<style scoped>
.fd-popover {
  position: relative;
  display: inline-flex;
}

.fd-popover__trigger {
  display: inline-flex;
}

.fd-popover__panel {
  position: absolute;
  outline: none;
  z-index: var(--fd-z-dropdown);
  padding: var(--fd-space-2);
  border: 1px solid var(--fd-border);
  border-radius: var(--fd-radius-panel);
  background: var(--fd-surface-raised);
  box-shadow: var(--fd-shadow-popover);
  max-width: calc(100vw - 24px);
}

.fd-popover__panel--bottom {
  top: calc(100% + 6px);
}

.fd-popover__panel--top {
  bottom: calc(100% + 6px);
}

.fd-popover__panel--end {
  right: 0;
}

.fd-popover__panel--start {
  left: 0;
}

.fd-popover-enter-active,
.fd-popover-leave-active {
  transition:
    opacity var(--fd-motion-hover) var(--fd-ease-standard),
    transform var(--fd-motion-hover) var(--fd-ease-standard);
}

.fd-popover-enter-from,
.fd-popover-leave-to {
  opacity: 0;
  transform: translateY(-2px);
}
</style>
