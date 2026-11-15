<script setup lang="ts">
import type { Component } from 'vue'
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRouter } from 'vue-router'

import FdBadge from '@/components/ui/FdBadge.vue'
import FdButton from '@/components/ui/FdButton.vue'
import FdEmptyState from '@/components/ui/FdEmptyState.vue'

import SettingsSection from './SettingsSection.vue'

const props = defineProps<{
  section: 'git' | 'integrations' | 'sync'
  icon: Component
  stage: number
}>()

const { t } = useI18n()
const router = useRouter()

const title = computed(() => t(`settings.${props.section}.title`))
const description = computed(() => t(`settings.${props.section}.description`))
const plannedTitle = computed(() => t(`settings.${props.section}.plannedTitle`))
const plannedBody = computed(() => t(`settings.${props.section}.plannedBody`))
</script>

<template>
  <SettingsSection :title="title" :description="description">
    <template #actions>
      <FdBadge tone="neutral">{{ t('common.stage', { stage }) }}</FdBadge>
    </template>
    <FdEmptyState :icon="icon" :title="plannedTitle" :body="plannedBody">
      <FdButton variant="secondary" @click="router.push('/settings/storage')">
        {{ t('settings.placeholder.openStorage') }}
      </FdButton>
    </FdEmptyState>
  </SettingsSection>
</template>
