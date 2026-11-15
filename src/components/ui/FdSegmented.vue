<script setup lang="ts">
import { computed, ref } from 'vue'

export interface SegmentedOption {
  value: string
  label: string
  disabled?: boolean
}

const props = withDefaults(
  defineProps<{
    options: SegmentedOption[]
    label: string
    disabled?: boolean
  }>(),
  { disabled: false },
)

const model = defineModel<string>({ required: true })
const buttons = ref<Array<HTMLButtonElement | null>>([])

const activeIndex = computed(() =>
  Math.max(
    0,
    props.options.findIndex((option) => option.value === model.value),
  ),
)

function setButton(element: unknown, index: number): void {
  buttons.value[index] = element as HTMLButtonElement | null
}

function select(option: SegmentedOption): void {
  if (!props.disabled && !option.disabled) {
    model.value = option.value
  }
}

function move(index: number, direction: 1 | -1): void {
  const count = props.options.length
  if (count === 0) {
    return
  }
  let next = index
  for (let step = 0; step < count; step += 1) {
    next = (next + direction + count) % count
    if (!props.options[next]?.disabled) {
      const option = props.options[next]
      if (option) {
        model.value = option.value
        buttons.value[next]?.focus()
      }
      return
    }
  }
}

function moveTo(index: number): void {
  const option = props.options[index]
  if (option && !option.disabled) {
    model.value = option.value
    buttons.value[index]?.focus()
  }
}

function onKeydown(event: KeyboardEvent, index: number): void {
  if (event.key === 'ArrowRight' || event.key === 'ArrowDown') {
    event.preventDefault()
    move(index, 1)
  } else if (event.key === 'ArrowLeft' || event.key === 'ArrowUp') {
    event.preventDefault()
    move(index, -1)
  } else if (event.key === 'Home') {
    event.preventDefault()
    moveTo(0)
  } else if (event.key === 'End') {
    event.preventDefault()
    moveTo(props.options.length - 1)
  }
}
</script>

<template>
  <div
    class="fd-segmented"
    role="radiogroup"
    :aria-label="label"
    :class="{ 'fd-segmented--disabled': disabled }"
  >
    <button
      v-for="(option, index) in options"
      :key="option.value"
      :ref="(element) => setButton(element, index)"
      type="button"
      role="radio"
      :aria-checked="model === option.value"
      :disabled="disabled || option.disabled"
      :tabindex="index === activeIndex ? 0 : -1"
      class="fd-segmented__option"
      :class="{ 'fd-segmented__option--active': model === option.value }"
      @click="select(option)"
      @keydown="onKeydown($event, index)"
    >
      {{ option.label }}
    </button>
  </div>
</template>

<style scoped>
.fd-segmented {
  display: inline-flex;
  gap: 2px;
  padding: 2px;
  border: 1px solid var(--fd-border);
  border-radius: var(--fd-radius-control);
  background: var(--fd-surface);
}

.fd-segmented__option {
  height: 26px;
  padding: 0 10px;
  border: 0;
  border-radius: 4px;
  background: transparent;
  color: var(--fd-text-secondary);
  font-size: var(--fd-text-12);
  font-weight: var(--fd-weight-medium);
  white-space: nowrap;
  transition:
    background-color var(--fd-motion-hover) var(--fd-ease-standard),
    color var(--fd-motion-hover) var(--fd-ease-standard);
}

.fd-segmented__option:hover:not(:disabled) {
  background: var(--fd-surface-hover);
  color: var(--fd-text);
}

.fd-segmented__option--active {
  background: var(--fd-surface-selected);
  color: var(--fd-text);
}

.fd-segmented__option:disabled {
  cursor: not-allowed;
  opacity: 0.5;
}

.fd-segmented--disabled {
  opacity: 0.6;
}
</style>
