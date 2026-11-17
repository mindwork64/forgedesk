<script setup lang="ts">
import { ChevronDown, Plus, Search, Settings } from '@lucide/vue'
import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRouter } from 'vue-router'

import { useGithubAuthStore } from '@/stores/githubAuth'
import { useUiStore } from '@/stores/ui'

import FdIconButton from '../ui/FdIconButton.vue'
import FdKbd from '../ui/FdKbd.vue'
import FdPopover from '../ui/FdPopover.vue'
import Breadcrumbs from './Breadcrumbs.vue'
import LocalStatusDetails from './LocalStatusDetails.vue'

const ui = useUiStore()
const auth = useGithubAuthStore()
const router = useRouter()
const { t } = useI18n()
const syncOpen = ref(false)
const githubStatus = computed(() => {
  if (auth.connectionLoading || !auth.connectionChecked) return t('status.githubChecking')
  if (auth.connected) return t('status.githubConnected', { login: auth.connection?.login ?? '' })
  if (auth.storageUnavailable) return t('status.githubStorageUnavailable')
  if (auth.connectionCheckFailed) return t('status.githubCheckFailed')
  return t('status.githubDisconnected')
})
const emit = defineEmits<{ (event: 'quick-create'): void }>()
</script>

<template>
  <header class="fd-header" data-testid="app-header">
    <div class="fd-header__left">
      <Breadcrumbs />
    </div>

    <div class="fd-header__right">
      <button
        type="button"
        class="fd-header__search"
        :aria-label="t('header.search')"
        @click="ui.openCommandPalette()"
      >
        <Search :size="14" aria-hidden="true" />
        <span class="fd-header__search-label">{{ t('header.searchPlaceholder') }}</span>
        <span class="fd-header__search-kbd">
          <FdKbd :keys="['Ctrl', 'K']" />
        </span>
      </button>

      <FdIconButton
        :label="t('header.quickCreate')"
        @click="emit('quick-create')"
      >
        <Plus :size="16" />
      </FdIconButton>

      <FdPopover
        v-model:open="syncOpen"
        :width="320"
        align="end"
        placement="bottom"
        :aria-label="t('status.details')"
      >
        <template #trigger="{ toggle }">
          <button
            type="button"
            class="fd-header__sync"
            :class="{ 'fd-header__sync--open': syncOpen }"
            :aria-expanded="syncOpen"
            :aria-label="t('status.details')"
            aria-haspopup="dialog"
            @click="toggle"
          >
            <span class="fd-header__sync-dot" aria-hidden="true" />
            <span class="fd-header__sync-label">{{ githubStatus }}</span>
            <ChevronDown :size="12" aria-hidden="true" />
          </button>
        </template>
        <LocalStatusDetails />
      </FdPopover>

      <FdIconButton
        :label="t('header.settings')"
        @click="router.push('/settings/general')"
      >
        <Settings :size="16" />
      </FdIconButton>
    </div>
  </header>
</template>

<style scoped>
.fd-header {
  display: flex;
  flex: 0 0 var(--fd-header-height);
  align-items: center;
  justify-content: space-between;
  gap: var(--fd-space-3);
  height: var(--fd-header-height);
  padding: 0 var(--fd-content-pad);
  border-bottom: 1px solid var(--fd-border);
  background: var(--fd-bg);
}

.fd-header__left {
  display: flex;
  flex: 1;
  align-items: center;
  min-width: 0;
}

.fd-header__right {
  display: flex;
  flex: none;
  align-items: center;
  gap: 4px;
}

.fd-header__search {
  display: flex;
  align-items: center;
  gap: 6px;
  width: 240px;
  height: 28px;
  padding: 0 8px;
  margin-right: 4px;
  border: 1px solid var(--fd-border);
  border-radius: var(--fd-radius-control);
  background: var(--fd-surface);
  color: var(--fd-text-muted);
  text-align: left;
  transition:
    background-color var(--fd-motion-hover) var(--fd-ease-standard),
    border-color var(--fd-motion-hover) var(--fd-ease-standard);
}

.fd-header__search:hover {
  border-color: var(--fd-text-muted);
  background: var(--fd-surface-hover);
}

.fd-header__search-label {
  flex: 1;
  overflow: hidden;
  font-size: var(--fd-text-12);
  text-overflow: ellipsis;
  white-space: nowrap;
}

.fd-header__search-kbd {
  display: inline-flex;
  flex: none;
}

.fd-header__sync {
  display: flex;
  align-items: center;
  gap: 6px;
  height: 28px;
  padding: 0 8px;
  border: 1px solid transparent;
  border-radius: var(--fd-radius-control);
  background: transparent;
  color: var(--fd-text-secondary);
  font-size: var(--fd-text-12);
}

.fd-header__sync:hover,
.fd-header__sync--open {
  background: var(--fd-surface-hover);
  color: var(--fd-text);
}

.fd-header__sync-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--fd-text-muted);
}

@media (max-width: 1279px) {
  .fd-header__search {
    width: 200px;
  }

  .fd-header__search-kbd {
    display: none;
  }
}

@media (max-width: 1140px) {
  .fd-header__sync-label {
    display: none;
  }

  .fd-header__search {
    width: 180px;
  }
}
</style>
