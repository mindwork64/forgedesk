<script setup lang="ts">
import { Keyboard, Settings, ShieldCheck } from '@lucide/vue'
import { useI18n } from 'vue-i18n'
import { useRouter } from 'vue-router'

import PageHeader from '@/components/layout/PageHeader.vue'
import FdButton from '@/components/ui/FdButton.vue'
import { useAppStore } from '@/stores/app'
import { useUiStore } from '@/stores/ui'

const app = useAppStore()
const ui = useUiStore()
const router = useRouter()
const { t } = useI18n()
</script>

<template>
  <div class="fd-page" data-testid="page-content">
    <PageHeader
      :title="t('pages.about.title')"
      :description="t('pages.about.description')"
    />

    <div class="fd-about">
      <section class="fd-about__intro">
        <h2 class="fd-about__heading">{{ t('about.heading') }}</h2>
        <p class="fd-about__body">{{ t('about.body') }}</p>
      </section>

      <p class="fd-about__notice">
        <ShieldCheck :size="15" aria-hidden="true" />
        <span>{{ t('about.noNetwork') }}</span>
      </p>

      <dl class="fd-about__grid">
        <dt>{{ t('about.version') }}</dt>
        <dd class="fd-mono">v{{ app.appVersion || '—' }}</dd>
        <dt>{{ t('about.schema') }}</dt>
        <dd class="fd-mono">v{{ app.schemaVersion }}</dd>
        <dt>{{ t('about.stack') }}</dt>
        <dd>{{ t('about.stackValue') }}</dd>
        <dt>{{ t('about.platforms') }}</dt>
        <dd>{{ t('about.platformsValue') }}</dd>
        <dt>{{ t('about.license') }}</dt>
        <dd>{{ t('about.licenseValue') }}</dd>
      </dl>

      <div class="fd-about__actions">
        <FdButton variant="secondary" @click="ui.openShortcuts()">
          <Keyboard :size="14" aria-hidden="true" />
          {{ t('about.shortcuts') }}
        </FdButton>
        <FdButton variant="secondary" @click="router.push('/settings/general')">
          <Settings :size="14" aria-hidden="true" />
          {{ t('about.openSettings') }}
        </FdButton>
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

.fd-about {
  display: flex;
  flex-direction: column;
  gap: var(--fd-space-5);
  width: 100%;
  max-width: 720px;
  padding: 0 var(--fd-content-pad) var(--fd-space-7);
}

.fd-about__heading {
  font-size: var(--fd-text-16);
  font-weight: var(--fd-weight-semibold);
  line-height: var(--fd-line-tight);
}

.fd-about__body {
  margin-top: var(--fd-space-2);
  color: var(--fd-text-secondary);
  font-size: var(--fd-text-13);
  line-height: var(--fd-line-body);
}

.fd-about__notice {
  display: flex;
  align-items: flex-start;
  gap: var(--fd-space-2);
  padding: 10px 12px;
  border: 1px solid var(--fd-border-subtle);
  border-radius: var(--fd-radius-control);
  background: var(--fd-surface);
  color: var(--fd-text-secondary);
  font-size: var(--fd-text-12);
  line-height: var(--fd-line-body);
}

.fd-about__notice svg {
  flex: none;
  margin-top: 1px;
  color: var(--fd-success);
}

.fd-about__grid {
  display: grid;
  grid-template-columns: 180px minmax(0, 1fr);
  gap: 10px var(--fd-space-4);
  margin: 0;
  padding: var(--fd-space-4) 0;
  border-top: 1px solid var(--fd-border-subtle);
  border-bottom: 1px solid var(--fd-border-subtle);
}

.fd-about__grid dt {
  color: var(--fd-text-muted);
  font-size: var(--fd-text-12);
}

.fd-about__grid dd {
  margin: 0;
  color: var(--fd-text-secondary);
  font-size: var(--fd-text-12);
}

.fd-about__actions {
  display: flex;
  gap: var(--fd-space-2);
}
</style>
