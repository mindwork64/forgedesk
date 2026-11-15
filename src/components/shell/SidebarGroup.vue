<script setup lang="ts">
import { ChevronDown, ChevronRight } from '@lucide/vue'
import { useI18n } from 'vue-i18n'

import type { NavGroup } from '@/navigation'

const props = defineProps<{
  group: NavGroup
  collapsed: boolean
  open: boolean
}>()

const emit = defineEmits<{ (event: 'toggle'): void }>()
const { t } = useI18n()
</script>

<template>
  <section class="fd-sidebar-group" :class="{ 'fd-sidebar-group--collapsed': collapsed }">
    <button
      v-if="!collapsed"
      type="button"
      class="fd-sidebar-group__label"
      :aria-expanded="open"
      @click="emit('toggle')"
    >
      <component
        :is="open ? ChevronDown : ChevronRight"
        :size="12"
        aria-hidden="true"
      />
      <span>{{ t(group.labelKey) }}</span>
    </button>
    <span v-else class="fd-sidebar-group__divider" aria-hidden="true" />
    <div v-show="collapsed || open" class="fd-sidebar-group__items">
      <slot />
    </div>
  </section>
</template>

<style scoped>
.fd-sidebar-group {
  display: flex;
  flex-direction: column;
  gap: 1px;
  margin-top: 2px;
}

.fd-sidebar-group:first-child {
  margin-top: 2px;
}

.fd-sidebar-group__label {
  display: flex;
  align-items: center;
  gap: 4px;
  height: 20px;
  padding: 0 4px;
  border: 0;
  border-radius: 4px;
  background: transparent;
  color: var(--fd-text-muted);
  font-size: var(--fd-text-11);
  font-weight: var(--fd-weight-semibold);
  letter-spacing: 0.06em;
  text-transform: uppercase;
}

.fd-sidebar-group__label:hover {
  color: var(--fd-text-secondary);
}

.fd-sidebar-group__divider {
  display: block;
  height: 1px;
  margin: 2px 6px;
  background: var(--fd-border-subtle);
}

.fd-sidebar-group--collapsed:first-child .fd-sidebar-group__divider {
  display: none;
}

.fd-sidebar-group__items {
  display: flex;
  flex-direction: column;
  gap: 1px;
}
</style>
