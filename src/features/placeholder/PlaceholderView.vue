<script setup lang="ts">
import { Layers } from '@lucide/vue'
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRoute, useRouter } from 'vue-router'

import PageHeader from '@/components/layout/PageHeader.vue'
import FdButton from '@/components/ui/FdButton.vue'
import FdEmptyState from '@/components/ui/FdEmptyState.vue'
import { NAV_GROUPS, UTILITY_NAV } from '@/navigation'

const route = useRoute()
const router = useRouter()
const { t } = useI18n()

const title = computed(() =>
  route.meta.titleKey ? t(route.meta.titleKey) : t('app.name'),
)
const description = computed(() =>
  route.meta.descriptionKey ? t(route.meta.descriptionKey) : undefined,
)
const stage = computed(() => route.meta.stage ?? 1)

const icon = computed(() => {
  const item = [...NAV_GROUPS.flatMap((group) => group.items), ...UTILITY_NAV].find(
    (candidate) => candidate.path === route.path,
  )
  return item?.icon ?? Layers
})
</script>

<template>
  <div class="fd-page" data-testid="page-content">
    <PageHeader :title="title" :description="description" />
    <div class="fd-page__body">
      <FdEmptyState
        :icon="icon"
        :title="t('pages.placeholder.planned', { stage })"
        :body="t('pages.placeholder.body')"
      >
        <FdButton variant="secondary" @click="router.push('/settings/storage')">
          {{ t('pages.placeholder.openStorage') }}
        </FdButton>
      </FdEmptyState>
    </div>
  </div>
</template>

<style scoped>
.fd-page {
  display: flex;
  flex-direction: column;
  min-height: 100%;
}

.fd-page__body {
  display: flex;
  flex: 1;
  align-items: flex-start;
  justify-content: center;
  padding: var(--fd-space-7) var(--fd-content-pad);
}
</style>
