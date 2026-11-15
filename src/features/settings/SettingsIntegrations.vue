<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import { useRouter } from 'vue-router'

import FdButton from '@/components/ui/FdButton.vue'

import SettingsSection from './SettingsSection.vue'

const { t } = useI18n()
const router = useRouter()
</script>

<template>
  <SettingsSection
    :title="t('settings.integrations.title')"
    :description="t('settings.integrations.description')"
  >
    <p class="fd-integrations-note">{{ t('settings.integrations.body') }}</p>
    <div>
      <FdButton variant="primary" @click="router.push('/integrations')">
        {{ t('settings.integrations.open') }}
      </FdButton>
    </div>
  </SettingsSection>
</template>

<style scoped>
.fd-integrations-note {
  max-width: 620px;
  margin: 0 0 12px;
  color: var(--fd-text-secondary);
  font-size: var(--fd-text-12);
  line-height: var(--fd-line-body);
}
</style>
