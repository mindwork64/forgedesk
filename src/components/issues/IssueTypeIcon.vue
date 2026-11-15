<script setup lang="ts">
import {
  Bug,
  CircleDot,
  FlaskConical,
  Sparkles,
  TrendingUp,
  Wrench,
} from '@lucide/vue'
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'

import { issueTypeLabel } from '@/lib/issue'
import type { IssueType } from '@/types/ipc'

const props = defineProps<{ type: IssueType }>()
const { t } = useI18n()

const icon = computed(() => {
  switch (props.type) {
    case 'bug':
      return Bug
    case 'feature':
      return Sparkles
    case 'improvement':
      return TrendingUp
    case 'chore':
      return Wrench
    case 'research':
      return FlaskConical
    default:
      return CircleDot
  }
})

const label = computed(() => issueTypeLabel(props.type, t))
</script>

<template>
  <span class="fd-type-icon" :title="label" :aria-label="label" role="img">
    <component :is="icon" :size="14" aria-hidden="true" />
  </span>
</template>

<style scoped>
.fd-type-icon {
  display: inline-flex;
  align-items: center;
  color: var(--fd-text-muted);
}
</style>
