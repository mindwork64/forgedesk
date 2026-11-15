<script setup lang="ts">
import { RefreshCw } from '@lucide/vue'
import type { Component } from 'vue'
import { computed, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRoute, useRouter } from 'vue-router'

import PageHeader from '@/components/layout/PageHeader.vue'

import SettingsAdvanced from './SettingsAdvanced.vue'
import SettingsAppearance from './SettingsAppearance.vue'
import SettingsGeneral from './SettingsGeneral.vue'
import SettingsGit from './SettingsGit.vue'
import SettingsIntegrations from './SettingsIntegrations.vue'
import SettingsPlaceholder from './SettingsPlaceholder.vue'
import SettingsShortcuts from './SettingsShortcuts.vue'
import SettingsStorage from './SettingsStorage.vue'
import SettingsWorkspace from './SettingsWorkspace.vue'

const route = useRoute()
const router = useRouter()
const { t } = useI18n()

const sectionKeys = [
  'general',
  'appearance',
  'workspace',
  'storage',
  'git',
  'integrations',
  'sync',
  'shortcuts',
  'advanced',
] as const

type SectionKey = (typeof sectionKeys)[number]

const section = computed<SectionKey>(() => {
  const raw = route.params.section
  return typeof raw === 'string' && (sectionKeys as readonly string[]).includes(raw)
    ? (raw as SectionKey)
    : 'general'
})

watch(
  section,
  (value) => {
    if (route.params.section !== value) {
      void router.replace(`/settings/${value}`)
    }
  },
  { immediate: true },
)

const componentMap: Partial<Record<SectionKey, Component>> = {
  general: SettingsGeneral,
  appearance: SettingsAppearance,
  workspace: SettingsWorkspace,
  storage: SettingsStorage,
  git: SettingsGit,
  integrations: SettingsIntegrations,
  shortcuts: SettingsShortcuts,
  advanced: SettingsAdvanced,
}

const placeholders: Partial<
  Record<SectionKey, { section: 'git' | 'integrations' | 'sync'; icon: Component; stage: number }>
> = {
  sync: { section: 'sync', icon: RefreshCw, stage: 4 },
}
</script>

<template>
  <div class="fd-page" data-testid="page-content">
    <PageHeader
      :title="t('settings.title')"
      :description="t('settings.subtitle')"
    />

    <div class="fd-settings-layout">
      <nav class="fd-settings-nav" :aria-label="t('settings.title')">
        <RouterLink
          v-for="key in sectionKeys"
          :key="key"
          :to="`/settings/${key}`"
          class="fd-settings-nav__item"
          :class="{ 'fd-settings-nav__item--active': section === key }"
          :aria-current="section === key ? 'page' : undefined"
        >
          {{ t(`settings.sections.${key}`) }}
        </RouterLink>
      </nav>

      <div class="fd-settings-content" data-testid="settings-content">
        <component :is="componentMap[section]" v-if="componentMap[section]" />
        <SettingsPlaceholder
          v-else-if="placeholders[section]"
          :section="placeholders[section]!.section"
          :icon="placeholders[section]!.icon"
          :stage="placeholders[section]!.stage"
        />
      </div>
    </div>
  </div>
</template>

<style scoped>
.fd-page {
  display: flex;
  flex-direction: column;
  min-height: 100%;
}

.fd-settings-layout {
  display: grid;
  grid-template-columns: 200px minmax(0, 1fr);
  gap: var(--fd-space-7);
  align-items: start;
  padding: 0 var(--fd-content-pad) var(--fd-space-7);
}

.fd-settings-nav {
  position: sticky;
  top: 0;
  display: flex;
  flex-direction: column;
  gap: 1px;
  align-self: start;
  padding-top: 4px;
}

.fd-settings-nav__item {
  display: flex;
  align-items: center;
  height: 30px;
  padding: 0 10px;
  border-radius: var(--fd-radius-control);
  color: var(--fd-text-secondary);
  font-size: var(--fd-text-13);
  text-decoration: none;
}

.fd-settings-nav__item:hover {
  background: var(--fd-surface-hover);
  color: var(--fd-text);
  text-decoration: none;
}

.fd-settings-nav__item--active {
  background: var(--fd-surface-selected);
  color: var(--fd-text);
  font-weight: var(--fd-weight-medium);
}

.fd-settings-content {
  width: 100%;
  max-width: 760px;
  padding-top: 4px;
}

@media (max-width: 1199px) {
  .fd-settings-layout {
    grid-template-columns: 170px minmax(0, 1fr);
    gap: var(--fd-space-6);
  }
}
</style>
