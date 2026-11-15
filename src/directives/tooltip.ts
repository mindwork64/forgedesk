import type { Directive } from 'vue'

export interface TooltipOptions {
  text: string
  placement?: 'top' | 'right' | 'bottom' | 'left'
}

export type TooltipValue = string | TooltipOptions | null | undefined | false

let tooltipEl: HTMLDivElement | null = null
let activeTarget: HTMLElement | null = null

function ensureElement(): HTMLDivElement {
  if (tooltipEl) {
    return tooltipEl
  }
  const element = document.createElement('div')
  element.className = 'fd-tooltip'
  element.id = `fd-tooltip-${Math.random().toString(36).slice(2, 8)}`
  element.setAttribute('role', 'tooltip')
  document.body.appendChild(element)
  tooltipEl = element
  return element
}

function normalise(value: TooltipValue): TooltipOptions | null {
  if (!value) {
    return null
  }
  if (typeof value === 'string') {
    return value.trim() ? { text: value } : null
  }
  return value.text.trim() ? value : null
}

function position(target: HTMLElement, placement: TooltipOptions['placement']): void {
  const element = ensureElement()
  const rect = target.getBoundingClientRect()
  const margin = 8
  const tooltipRect = element.getBoundingClientRect()

  let left = 0
  let top = 0
  const resolved =
    placement ??
    (target.closest('.fd-sidebar--collapsed') ? 'right' : 'bottom')

  switch (resolved) {
    case 'right':
      left = rect.right + margin
      top = rect.top + rect.height / 2 - tooltipRect.height / 2
      break
    case 'left':
      left = rect.left - tooltipRect.width - margin
      top = rect.top + rect.height / 2 - tooltipRect.height / 2
      break
    case 'top':
      left = rect.left + rect.width / 2 - tooltipRect.width / 2
      top = rect.top - tooltipRect.height - margin
      break
    default:
      left = rect.left + rect.width / 2 - tooltipRect.width / 2
      top = rect.bottom + margin
      break
  }

  left = Math.min(
    Math.max(margin, left),
    window.innerWidth - tooltipRect.width - margin,
  )
  top = Math.min(
    Math.max(margin, top),
    window.innerHeight - tooltipRect.height - margin,
  )

  element.style.left = `${Math.round(left)}px`
  element.style.top = `${Math.round(top)}px`
}

function show(target: HTMLElement, value: TooltipValue): void {
  const options = normalise(value)
  if (!options) {
    return
  }
  const element = ensureElement()
  element.textContent = options.text
  element.style.visibility = 'hidden'
  element.style.opacity = '1'
  activeTarget = target
  position(target, options.placement)
  element.style.visibility = 'visible'
  target.setAttribute('aria-describedby', element.id)
}

function hide(): void {
  if (tooltipEl) {
    tooltipEl.style.visibility = 'hidden'
    tooltipEl.style.opacity = '0'
  }
  if (activeTarget) {
    activeTarget.removeAttribute('aria-describedby')
    activeTarget = null
  }
}

type State = {
  value: TooltipValue
  enter: () => void
  leave: () => void
  focus: () => void
  blur: () => void
  escape: (event: KeyboardEvent) => void
}

const states = new WeakMap<HTMLElement, State>()

export const tooltip: Directive<HTMLElement, TooltipValue> = {
  mounted(target, binding) {
    const bound: State = {
      value: binding.value,
      enter: () => show(target, bound.value),
      leave: hide,
      focus: () => show(target, bound.value),
      blur: hide,
      escape: (event) => {
        if (event.key === 'Escape') {
          hide()
        }
      },
    }
    states.set(target, bound)
    target.addEventListener('mouseenter', bound.enter)
    target.addEventListener('mouseleave', bound.leave)
    target.addEventListener('focus', bound.focus)
    target.addEventListener('blur', bound.blur)
    target.addEventListener('keydown', bound.escape)
    target.addEventListener('pointerdown', bound.leave)
    window.addEventListener('scroll', hide, true)
  },
  updated(target, binding) {
    const bound = states.get(target)
    if (bound) {
      bound.value = binding.value
    }
  },
  unmounted(target) {
    const bound = states.get(target)
    if (bound) {
      target.removeEventListener('mouseenter', bound.enter)
      target.removeEventListener('mouseleave', bound.leave)
      target.removeEventListener('focus', bound.focus)
      target.removeEventListener('blur', bound.blur)
      target.removeEventListener('keydown', bound.escape)
      target.removeEventListener('pointerdown', bound.leave)
      states.delete(target)
    }
    window.removeEventListener('scroll', hide, true)
    hide()
  },
}
