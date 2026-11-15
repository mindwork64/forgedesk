<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRoute } from 'vue-router'

import type { NavItem } from '@/navigation'

const props = defineProps<{
  item: NavItem
  collapsed: boolean
}>()

const route = useRoute()
const { t } = useI18n()

const active = computed(() => {
  if (props.item.prefix) {
    return (
      route.path === props.item.path ||
      route.path.startsWith(`${props.item.path}/`)
    )
  }
  return route.path === props.item.path
})

const label = computed(() => t(props.item.labelKey))
</script>

<template>
  <RouterLink
    v-tooltip="
      collapsed ? { text: label, placement: 'right' } : false
    "
    :to="item.path"
    class="fd-nav-item"
    :class="{
      'fd-nav-item--active': active,
      'fd-nav-item--collapsed': collapsed,
    }"
    :aria-current="active ? 'page' : undefined"
    :aria-label="collapsed ? label : undefined"
  >
    <component
      :is="item.icon"
      :size="16"
      class="fd-nav-item__icon"
      aria-hidden="true"
    />
    <span v-if="!collapsed" class="fd-nav-item__label">{{ label }}</span>
  </RouterLink>
</template>

<style scoped>
.fd-nav-item {
  position: relative;
  display: flex;
  align-items: center;
  gap: 10px;
  height: var(--fd-nav-item-height);
  padding: 0 8px;
  border-radius: var(--fd-radius-control);
  color: var(--fd-text-secondary);
  text-decoration: none;
  transition:
    background-color var(--fd-motion-hover) var(--fd-ease-standard),
    color var(--fd-motion-hover) var(--fd-ease-standard);
}

.fd-nav-item:hover {
  background: var(--fd-surface-hover);
  color: var(--fd-text);
  text-decoration: none;
}

.fd-nav-item:focus-visible {
  outline: 2px solid var(--fd-accent);
  outline-offset: -2px;
}

.fd-nav-item--active {
  background: var(--fd-accent-soft);
  color: var(--fd-accent);
}

.fd-nav-item--active:hover {
  background: var(--fd-accent-soft);
  color: var(--fd-accent);
}

.fd-nav-item--active::before {
  content: '';
  position: absolute;
  left: -8px;
  top: 6px;
  bottom: 6px;
  width: 2px;
  border-radius: 1px;
  background: var(--fd-accent);
}

.fd-nav-item--collapsed {
  justify-content: center;
  width: 40px;
  padding: 0;
}

.fd-nav-item__icon {
  flex: none;
}

.fd-nav-item__label {
  overflow: hidden;
  font-size: var(--fd-text-13);
  font-weight: var(--fd-weight-medium);
  text-overflow: ellipsis;
  white-space: nowrap;
}
</style>
