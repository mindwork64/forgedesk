<script setup lang="ts">
import { Check, Pin, PinOff, Trash2 } from '@lucide/vue'
import { useI18n } from 'vue-i18n'

import FdButton from '@/components/ui/FdButton.vue'
import FdIconButton from '@/components/ui/FdIconButton.vue'
import type { SavedView } from '@/types/ipc'

const props = defineProps<{
  views: SavedView[]
  activeId: string | null
}>()

const emit = defineEmits<{
  (event: 'apply', view: SavedView): void
  (event: 'pin', view: SavedView): void
  (event: 'delete', view: SavedView): void
  (event: 'save'): void
}>()

const { t } = useI18n()

function isActive(view: SavedView): boolean {
  return props.activeId === view.id
}
</script>

<template>
  <div class="fd-saved-views">
    <div class="fd-saved-views__head">
      <p class="fd-saved-views__title">{{ t('views.savedView') }}</p>
      <FdButton variant="ghost" size="sm" @click="emit('save')">
        {{ t('views.save') }}
      </FdButton>
    </div>

    <p v-if="views.length === 0" class="fd-saved-views__empty">
      {{ t('views.empty') }}
    </p>

    <div v-else class="fd-saved-views__list">
      <div
        v-for="view in views"
        :key="view.id"
        class="fd-saved-views__row"
        :class="{ 'fd-saved-views__row--active': isActive(view) }"
      >
        <button
          type="button"
          class="fd-saved-views__apply"
          :aria-current="isActive(view) ? 'true' : undefined"
          @click="emit('apply', view)"
        >
          <Check v-if="isActive(view)" :size="12" aria-hidden="true" />
          <span v-else class="fd-saved-views__spacer" aria-hidden="true" />
          <span class="fd-saved-views__name">{{ view.name }}</span>
        </button>
        <FdIconButton
          :label="view.isPinned ? t('views.unpin') : t('views.pin')"
          @click.stop="emit('pin', view)"
        >
          <PinOff v-if="view.isPinned" :size="13" />
          <Pin v-else :size="13" />
        </FdIconButton>
        <FdIconButton
          :label="t('views.delete')"
          @click.stop="emit('delete', view)"
        >
          <Trash2 :size="13" />
        </FdIconButton>
      </div>
    </div>
  </div>
</template>

<style scoped>
.fd-saved-views {
  display: flex;
  flex-direction: column;
  gap: var(--fd-space-2);
  width: 300px;
  padding: var(--fd-space-2);
}

.fd-saved-views__head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--fd-space-2);
  padding-left: 4px;
}

.fd-saved-views__title {
  margin: 0;
  font-size: var(--fd-text-12);
  font-weight: 600;
}

.fd-saved-views__empty {
  margin: 0;
  padding: var(--fd-space-2) 4px;
  color: var(--fd-text-muted);
  font-size: var(--fd-text-11);
}

.fd-saved-views__list {
  display: flex;
  flex-direction: column;
  gap: 1px;
}

.fd-saved-views__row {
  display: flex;
  align-items: center;
  gap: 2px;
  min-height: 32px;
  padding: 0 2px;
  border-radius: var(--fd-radius-control);
}

.fd-saved-views__row:hover,
.fd-saved-views__row--active {
  background: var(--fd-surface-hover);
}

.fd-saved-views__apply {
  display: flex;
  align-items: center;
  gap: 6px;
  flex: 1;
  min-width: 0;
  height: 30px;
  padding: 0 4px;
  border: 0;
  background: transparent;
  color: var(--fd-text-secondary);
  text-align: left;
  font-size: var(--fd-text-12);
  cursor: pointer;
}

.fd-saved-views__row--active .fd-saved-views__apply {
  color: var(--fd-text-primary);
}

.fd-saved-views__spacer {
  width: 12px;
  flex: none;
}

.fd-saved-views__name {
  overflow: hidden;
  white-space: nowrap;
  text-overflow: ellipsis;
}
</style>
