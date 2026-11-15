<script setup lang="ts">
import { ChevronRight } from '@lucide/vue'
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRoute } from 'vue-router'

const route = useRoute()
const { t, te } = useI18n()

interface Crumb {
  label: string
  to?: string
}

const SETTINGS_SECTIONS = new Set([
  'general',
  'appearance',
  'workspace',
  'storage',
  'git',
  'integrations',
  'sync',
  'shortcuts',
  'advanced',
])

const crumbs = computed<Crumb[]>(() => {
  if (route.path.startsWith('/settings')) {
    const raw = route.params.section
    const section =
      typeof raw === 'string' && SETTINGS_SECTIONS.has(raw) ? raw : 'general'
    return [
      { label: t('nav.settings'), to: '/settings/general' },
      {
        label: te(`settings.sections.${section}`)
          ? t(`settings.sections.${section}`)
          : section,
      },
    ]
  }
  const key = route.meta.titleKey
  return [{ label: key ? t(key) : t('app.name') }]
})
</script>

<template>
  <nav class="fd-breadcrumbs" :aria-label="t('header.breadcrumbs')">
    <template v-for="(crumb, index) in crumbs" :key="`${crumb.label}-${index}`">
      <ChevronRight
        v-if="index > 0"
        :size="12"
        class="fd-breadcrumbs__separator"
        aria-hidden="true"
      />
      <RouterLink
        v-if="crumb.to && index < crumbs.length - 1"
        :to="crumb.to"
        class="fd-breadcrumbs__link"
      >
        {{ crumb.label }}
      </RouterLink>
      <span v-else class="fd-breadcrumbs__current" :aria-current="index === crumbs.length - 1 ? 'page' : undefined">
        {{ crumb.label }}
      </span>
    </template>
  </nav>
</template>

<style scoped>
.fd-breadcrumbs {
  display: flex;
  align-items: center;
  gap: 4px;
  min-width: 0;
  overflow: hidden;
}

.fd-breadcrumbs__link {
  color: var(--fd-text-secondary);
  font-size: var(--fd-text-13);
  font-weight: var(--fd-weight-medium);
  white-space: nowrap;
}

.fd-breadcrumbs__link:hover {
  color: var(--fd-text);
  text-decoration: none;
}

.fd-breadcrumbs__current {
  overflow: hidden;
  color: var(--fd-text);
  font-size: var(--fd-text-13);
  font-weight: var(--fd-weight-medium);
  text-overflow: ellipsis;
  white-space: nowrap;
}

.fd-breadcrumbs__separator {
  flex: none;
  color: var(--fd-text-muted);
}
</style>
