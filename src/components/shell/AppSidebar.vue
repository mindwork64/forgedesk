<script setup lang="ts">
import { PanelLeftClose } from '@lucide/vue'
import { useI18n } from 'vue-i18n'

import { NAV_GROUPS, PINNED_VIEWS, UTILITY_NAV, WORKSPACE_MARK } from '@/navigation'
import { useAppStore } from '@/stores/app'
import { useUiStore } from '@/stores/ui'

import FdIconButton from '../ui/FdIconButton.vue'
import SidebarGroup from './SidebarGroup.vue'
import SidebarNavItem from './SidebarNavItem.vue'

const props = defineProps<{
  collapsed: boolean
  canToggle: boolean
}>()

const emit = defineEmits<{ (event: 'toggle'): void }>()

const app = useAppStore()
const ui = useUiStore()
const { t } = useI18n()
</script>

<template>
  <aside
    class="fd-sidebar"
    :class="{ 'fd-sidebar--collapsed': collapsed }"
    data-testid="app-sidebar"
  >
    <div class="fd-sidebar__workspace">
      <button
        v-if="props.collapsed"
        v-tooltip="
          canToggle
            ? { text: t('nav.expand'), placement: 'right' }
            : { text: app.workspace?.name ?? t('app.workspaceFallback'), placement: 'right' }
        "
        type="button"
        class="fd-sidebar__mark-button"
        :aria-label="canToggle ? t('nav.expand') : (app.workspace?.name ?? t('app.workspaceFallback'))"
        :disabled="!canToggle"
        @click="emit('toggle')"
      >
        <component :is="WORKSPACE_MARK" :size="18" />
      </button>
      <template v-else>
        <RouterLink to="/my-issues" aria-current-value="false" class="fd-sidebar__identity">
          <span class="fd-sidebar__mark">
            <component :is="WORKSPACE_MARK" :size="18" aria-hidden="true" />
          </span>
          <span class="fd-sidebar__identity-text">
            <span class="fd-sidebar__wordmark">{{ t('app.wordmark') }}</span>
            <span class="fd-sidebar__workspace-name">
              {{ app.workspace?.name ?? t('app.workspaceFallback') }}
            </span>
          </span>
        </RouterLink>
        <FdIconButton
          :label="t('nav.collapse')"
          :disabled="!canToggle"
          :tooltip="canToggle ? t('nav.collapse') : t('nav.resizeUnavailable')"
          @click="emit('toggle')"
        >
          <PanelLeftClose :size="16" />
        </FdIconButton>
      </template>
    </div>

    <nav class="fd-sidebar__scroll" :aria-label="t('nav.mainNavigation')">
      <SidebarGroup
        v-for="group in NAV_GROUPS"
        :key="group.id"
        :group="group"
        :collapsed="collapsed"
        :open="ui.groups[group.id]"
        @toggle="ui.toggleGroup(group.id)"
      >
        <SidebarNavItem
          v-for="item in group.items"
          :key="item.key"
          :item="item"
          :collapsed="collapsed"
        />
        <template v-if="group.id === 'workspace' && !collapsed && PINNED_VIEWS.length > 0">
          <div class="fd-sidebar__pinned">
            <span class="fd-sidebar__pinned-label">{{ t('nav.pinnedViews') }}</span>
            <RouterLink
              v-for="view in PINNED_VIEWS"
              :key="view.id"
              :to="view.path"
              class="fd-sidebar__pinned-link"
            >
              {{ view.name }}
            </RouterLink>
          </div>
        </template>
      </SidebarGroup>
    </nav>

    <nav class="fd-sidebar__bottom" :aria-label="t('nav.utilityNavigation')">
      <SidebarNavItem
        v-for="item in UTILITY_NAV"
        :key="item.key"
        :item="item"
        :collapsed="collapsed"
      />
    </nav>
  </aside>
</template>

<style scoped>
.fd-sidebar {
  display: flex;
  flex: 0 0 var(--fd-sidebar-width);
  flex-direction: column;
  width: var(--fd-sidebar-width);
  min-height: 0;
  padding: 8px var(--fd-sidebar-pad) 6px;
  border-right: 1px solid var(--fd-border);
  background: var(--fd-sidebar);
  transition: width var(--fd-motion-drawer) var(--fd-ease-standard);
}

.fd-sidebar--collapsed {
  flex-basis: var(--fd-sidebar-collapsed-width);
  width: var(--fd-sidebar-collapsed-width);
  padding-right: 12px;
  padding-left: 12px;
}

.fd-sidebar__workspace {
  display: flex;
  align-items: center;
  gap: 6px;
  height: 36px;
  padding: 0 4px;
  margin-bottom: 4px;
}

.fd-sidebar--collapsed .fd-sidebar__workspace {
  justify-content: center;
  padding: 0;
}

.fd-sidebar__mark-button {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  padding: 0;
  border: 1px solid var(--fd-border);
  border-radius: var(--fd-radius-control);
  background: var(--fd-surface);
  color: var(--fd-accent);
}

.fd-sidebar__mark-button:hover:not(:disabled) {
  background: var(--fd-surface-hover);
}

.fd-sidebar__mark-button:disabled {
  cursor: default;
}

.fd-sidebar__identity {
  display: flex;
  flex: 1;
  align-items: center;
  gap: 8px;
  min-width: 0;
  color: inherit;
  text-decoration: none;
}

.fd-sidebar__identity:hover {
  text-decoration: none;
}

.fd-sidebar__mark {
  display: flex;
  flex: none;
  align-items: center;
  justify-content: center;
  width: 26px;
  height: 26px;
  border-radius: var(--fd-radius-control);
  background: var(--fd-accent-soft);
  color: var(--fd-accent);
}

.fd-sidebar__identity-text {
  display: flex;
  flex-direction: column;
  min-width: 0;
  line-height: var(--fd-line-tight);
}

.fd-sidebar__wordmark {
  overflow: hidden;
  color: var(--fd-text);
  font-size: var(--fd-text-13);
  font-weight: var(--fd-weight-semibold);
  text-overflow: ellipsis;
  white-space: nowrap;
}

.fd-sidebar__workspace-name {
  overflow: hidden;
  color: var(--fd-text-muted);
  font-size: var(--fd-text-11);
  text-overflow: ellipsis;
  white-space: nowrap;
}

.fd-sidebar__scroll {
  flex: 1 1 auto;
  min-height: 0;
  padding: 0 4px;
  margin: 0 -4px;
  overflow-x: hidden;
  overflow-y: auto;
}

.fd-sidebar--collapsed .fd-sidebar__scroll {
  display: flex;
  flex-direction: column;
  align-items: center;
  width: 100%;
  padding: 0;
  margin: 0;
}

.fd-sidebar__bottom {
  display: flex;
  flex-direction: column;
  gap: 2px;
  padding: 6px 4px 0;
  margin-top: 4px;
  border-top: 1px solid var(--fd-border-subtle);
}

.fd-sidebar--collapsed .fd-sidebar__bottom {
  align-items: center;
  width: 100%;
  padding: 6px 0 0;
}

.fd-sidebar__pinned {
  display: flex;
  flex-direction: column;
  gap: 1px;
  padding: 2px 0 2px 26px;
}

.fd-sidebar__pinned-label {
  color: var(--fd-text-muted);
  font-size: var(--fd-text-11);
  line-height: 20px;
}

.fd-sidebar__pinned-link {
  overflow: hidden;
  padding: 2px 6px;
  border-radius: 4px;
  color: var(--fd-text-secondary);
  font-size: var(--fd-text-12);
  text-decoration: none;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.fd-sidebar__pinned-link:hover {
  background: var(--fd-surface-hover);
  color: var(--fd-text);
  text-decoration: none;
}
</style>
