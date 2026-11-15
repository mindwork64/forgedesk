<script setup lang="ts">
import { GitBranch, ShieldCheck } from '@lucide/vue'
import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRouter } from 'vue-router'

import FdButton from '@/components/ui/FdButton.vue'
import FdField from '@/components/ui/FdField.vue'
import FdInput from '@/components/ui/FdInput.vue'
import FdModal from '@/components/ui/FdModal.vue'
import FdSegmented from '@/components/ui/FdSegmented.vue'
import { toIpcError } from '@/lib/ipc'
import { useAppStore } from '@/stores/app'
import type {
  LocalePreference,
  ThemePreference,
} from '@/types/ipc'

const app = useAppStore()
const router = useRouter()
const { t } = useI18n()

const step = ref(app.workspace ? 2 : 1)
const name = ref(app.workspace?.name ?? 'My Workspace')
const locale = ref<string>(app.settings?.locale ?? 'system')
const theme = ref<string>(app.settings?.theme ?? 'dark')
const busy = ref(false)
const nameError = ref<string | null>(null)
const formError = ref<string | null>(null)

const open = computed({
  get: () => true,
  set: () => {
    // Onboarding is mandatory and cannot be dismissed.
  },
})

const localeOptions = computed(() => [
  { value: 'system', label: t('common.system') },
  { value: 'en', label: 'English' },
  { value: 'ru', label: 'Русский' },
])

const themeOptions = computed(() => [
  { value: 'dark', label: t('settings.appearance.themeDark') },
  { value: 'light', label: t('settings.appearance.themeLight') },
  { value: 'system', label: t('settings.appearance.themeSystem') },
])

async function submit(): Promise<void> {
  formError.value = null
  if (step.value === 1) {
    const trimmed = name.value.trim()
    if (!trimmed) {
      nameError.value = t('onboarding.workspaceRequired')
      return
    }
    nameError.value = null
    busy.value = true
    try {
      await app.createWorkspace(trimmed)
      step.value = 2
    } catch (error) {
      formError.value = toIpcError(error).message
    } finally {
      busy.value = false
    }
    return
  }

  if (step.value === 2) {
    busy.value = true
    const saved = await app.saveSettings({
      locale: locale.value as LocalePreference,
      theme: theme.value as ThemePreference,
    })
    busy.value = false
    if (saved) {
      step.value = 3
    } else {
      formError.value = t('onboarding.settingsError')
    }
    return
  }

  busy.value = true
  const completed = await app.completeOnboarding()
  busy.value = false
  if (completed) {
    await router.replace('/my-issues')
  } else {
    formError.value = t('onboarding.finishError')
  }
}
</script>

<template>
  <FdModal
    v-model="open"
    :title="t('onboarding.title')"
    size="md"
    persistent
  >
    <div class="fd-onboarding">
      <div class="fd-onboarding__progress">
        <div class="fd-onboarding__bars" aria-hidden="true">
          <span
            v-for="index in 3"
            :key="index"
            class="fd-onboarding__bar"
            :class="{ 'fd-onboarding__bar--active': index <= step }"
          />
        </div>
        <span class="fd-onboarding__step">
          {{ t('onboarding.step', { current: step, total: 3 }) }}
        </span>
      </div>

      <form class="fd-onboarding__form" @submit.prevent="submit">
        <template v-if="step === 1">
          <div class="fd-onboarding__intro">
            <h3 class="fd-onboarding__heading">{{ t('onboarding.workspaceTitle') }}</h3>
            <p class="fd-onboarding__description">
              {{ t('onboarding.workspaceDescription') }}
            </p>
          </div>
          <FdField
            :label="t('onboarding.workspaceName')"
            :error="nameError ?? undefined"
            required
          >
            <template #default="{ id, describedBy }">
              <FdInput
                :id="id"
                v-model="name"
                :aria-describedby="describedBy"
                :invalid="!!nameError"
                :maxlength="80"
                autofocus
                @update:model-value="nameError = null"
              />
            </template>
          </FdField>
        </template>

        <template v-else-if="step === 2">
          <div class="fd-onboarding__intro">
            <h3 class="fd-onboarding__heading">{{ t('onboarding.preferencesTitle') }}</h3>
            <p class="fd-onboarding__description">
              {{ t('onboarding.preferencesDescription') }}
            </p>
          </div>
          <div class="fd-onboarding__option">
            <div class="fd-onboarding__option-text">
              <span class="fd-onboarding__option-label">{{ t('onboarding.language') }}</span>
              <span class="fd-onboarding__option-hint">{{ t('settings.general.languageHint') }}</span>
            </div>
            <FdSegmented
              v-model="locale"
              :options="localeOptions"
              :label="t('onboarding.language')"
            />
          </div>
          <div class="fd-onboarding__option">
            <div class="fd-onboarding__option-text">
              <span class="fd-onboarding__option-label">{{ t('onboarding.theme') }}</span>
              <span class="fd-onboarding__option-hint">{{ t('settings.appearance.themeHint') }}</span>
            </div>
            <FdSegmented
              v-model="theme"
              :options="themeOptions"
              :label="t('onboarding.theme')"
            />
          </div>
        </template>

        <template v-else>
          <div class="fd-onboarding__intro">
            <h3 class="fd-onboarding__heading">{{ t('onboarding.connectTitle') }}</h3>
            <p class="fd-onboarding__description">
              {{ t('onboarding.connectDescription') }}
            </p>
          </div>
          <div class="fd-onboarding__provider">
            <span class="fd-onboarding__provider-icon">
              <GitBranch :size="20" aria-hidden="true" />
            </span>
            <div class="fd-onboarding__provider-text">
              <span class="fd-onboarding__provider-title">GitHub</span>
              <span class="fd-onboarding__provider-note">
                {{ t('onboarding.connectOptional') }}
              </span>
            </div>
          </div>
          <p class="fd-onboarding__secure">
            <ShieldCheck :size="14" aria-hidden="true" />
            {{ t('settings.integrations.localOnlyBody') }}
          </p>
        </template>

        <p v-if="formError" class="fd-onboarding__error" role="alert">
          {{ formError }}
        </p>
      </form>
    </div>

    <template #footer>
      <FdButton
        v-if="step === 3"
        variant="ghost"
        :disabled="busy"
        @click="step = 1"
      >
        {{ t('common.back') }}
      </FdButton>
      <FdButton
        :loading="busy"
        type="button"
        @click="submit"
      >
        <template v-if="step === 3">{{ t('onboarding.skip') }}</template>
        <template v-else>{{ t('onboarding.next') }}</template>
      </FdButton>
    </template>
  </FdModal>
</template>

<style scoped>
.fd-onboarding {
  display: flex;
  flex-direction: column;
  gap: var(--fd-space-4);
}

.fd-onboarding__progress {
  display: flex;
  align-items: center;
  gap: var(--fd-space-3);
}

.fd-onboarding__bars {
  display: flex;
  flex: 1;
  gap: 4px;
}

.fd-onboarding__bar {
  flex: 1;
  height: 3px;
  border-radius: 999px;
  background: var(--fd-border);
  transition: background-color var(--fd-motion-hover) var(--fd-ease-standard);
}

.fd-onboarding__bar--active {
  background: var(--fd-accent);
}

.fd-onboarding__step {
  flex: none;
  color: var(--fd-text-muted);
  font-size: var(--fd-text-11);
}

.fd-onboarding__form {
  display: flex;
  flex-direction: column;
  gap: var(--fd-space-4);
}

.fd-onboarding__intro {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.fd-onboarding__heading {
  font-size: var(--fd-text-14);
  font-weight: var(--fd-weight-semibold);
  line-height: var(--fd-line-tight);
}

.fd-onboarding__description {
  color: var(--fd-text-secondary);
  font-size: var(--fd-text-13);
  line-height: var(--fd-line-body);
}

.fd-onboarding__option {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: var(--fd-space-4);
  padding: var(--fd-space-3) 0;
  border-top: 1px solid var(--fd-border-subtle);
}

.fd-onboarding__option-text {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}

.fd-onboarding__option-label {
  font-size: var(--fd-text-13);
  font-weight: var(--fd-weight-medium);
}

.fd-onboarding__option-hint {
  max-width: 300px;
  color: var(--fd-text-muted);
  font-size: var(--fd-text-12);
  line-height: var(--fd-line-body);
}

.fd-onboarding__provider {
  display: flex;
  align-items: center;
  gap: var(--fd-space-3);
  padding: var(--fd-space-3);
  border: 1px solid var(--fd-border);
  border-radius: var(--fd-radius-panel);
  background: var(--fd-surface);
}

.fd-onboarding__provider-icon {
  display: flex;
  flex: none;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 36px;
  border-radius: var(--fd-radius-control);
  background: var(--fd-accent-soft);
  color: var(--fd-accent);
}

.fd-onboarding__provider-text {
  display: flex;
  flex: 1;
  flex-direction: column;
  min-width: 0;
}

.fd-onboarding__provider-title {
  font-size: var(--fd-text-13);
  font-weight: var(--fd-weight-medium);
}

.fd-onboarding__provider-note {
  color: var(--fd-text-muted);
  font-size: var(--fd-text-11);
  line-height: var(--fd-line-body);
}

.fd-onboarding__secure {
  display: flex;
  align-items: flex-start;
  gap: 6px;
  color: var(--fd-text-muted);
  font-size: var(--fd-text-11);
  line-height: var(--fd-line-body);
}

.fd-onboarding__error {
  padding: 8px 10px;
  border: 1px solid var(--fd-danger);
  border-radius: var(--fd-radius-control);
  background: var(--fd-danger-soft);
  color: var(--fd-danger);
  font-size: var(--fd-text-12);
}
</style>
