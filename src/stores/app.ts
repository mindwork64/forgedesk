import { defineStore } from 'pinia'
import { computed, ref, watch } from 'vue'

import { applyLocale, i18n, resolveLocale, resolveTheme } from '@/i18n'
import { api, toIpcError } from '@/lib/ipc'
import { applyResolvedTheme } from '@/lib/theme'
import type {
  AppSettings,
  BackupInfo,
  Bootstrap,
  IpcError,
  LocalePreference,
  RestoreResult,
  SettingsPatch,
  ThemePreference,
  Workspace,
} from '@/types/ipc'
import { useToastStore } from './toasts'

const t = (key: string, params?: Record<string, unknown>): string => {
  const translate = i18n.global.t as (
    key: string,
    params?: Record<string, unknown>,
  ) => string
  return params ? translate(key, params) : translate(key)
}

export const useAppStore = defineStore('app', () => {
  const data = ref<Bootstrap | null>(null)
  const loading = ref(true)
  const startupError = ref<IpcError | null>(null)
  const lastSavedAt = ref<number | null>(null)

  const toast = useToastStore()

  const systemPrefersDark = ref(true)
  if (
    typeof window !== 'undefined' &&
    typeof window.matchMedia === 'function'
  ) {
    const media = window.matchMedia('(prefers-color-scheme: dark)')
    systemPrefersDark.value = media.matches
    media.addEventListener('change', (event) => {
      systemPrefersDark.value = event.matches
    })
  }

  const settings = computed<AppSettings | null>(
    () => data.value?.settings ?? null,
  )
  const workspace = computed<Workspace | null>(
    () => data.value?.workspace ?? null,
  )
  const backups = computed<BackupInfo[]>(() => data.value?.backups ?? [])
  const appVersion = computed(() => data.value?.appVersion ?? '')
  const schemaVersion = computed(() => data.value?.schemaVersion ?? 0)
  const startupWarning = computed(() => data.value?.startupWarning ?? null)
  const resolvedLocale = computed(() =>
    resolveLocale(settings.value?.locale ?? 'system'),
  )
  const resolvedTheme = computed(() =>
    resolveTheme(settings.value?.theme ?? 'dark', systemPrefersDark.value),
  )
  const sidebarCollapsed = computed(
    () => settings.value?.sidebarCollapsed ?? false,
  )
  const needsOnboarding = computed(
    () =>
      data.value !== null &&
      (data.value.workspace === null ||
        !data.value.settings.onboardingCompleted),
  )

  watch(
    [resolvedTheme, resolvedLocale],
    ([theme, locale]) => {
      applyResolvedTheme(theme)
      applyLocale(locale)
    },
    { immediate: true },
  )

  async function load(): Promise<void> {
    loading.value = true
    startupError.value = null
    try {
      const next = await api.bootstrap()
      data.value = next
      lastSavedAt.value = null
      if (next.startupWarning) {
        toast.warning(t('status.autoBackup'), next.startupWarning)
      }
    } catch (error) {
      startupError.value = toIpcError(error)
      data.value = null
    } finally {
      loading.value = false
    }
  }

  // Settings writes are serialized. Without a queue, two quick changes can
  // return out of order and an older response would overwrite a newer value.
  let settingsQueue: Promise<boolean> = Promise.resolve(true)
  let settingsRevision = 0

  async function saveSettings(patch: SettingsPatch): Promise<boolean> {
    if (!data.value) {
      return false
    }
    const previous = data.value.settings
    data.value.settings = { ...previous, ...patch }
    const revision = ++settingsRevision

    const task = settingsQueue.then(async () => {
      try {
        const saved = await api.saveSettings(patch)
        if (data.value) {
          data.value.settings = saved
        }
        lastSavedAt.value = Date.now()
        return true
      } catch (error) {
        if (data.value) {
          if (revision === settingsRevision) {
            data.value.settings = previous
          } else {
            // A newer optimistic change exists; resync rather than clobber it.
            try {
              data.value = await api.bootstrap()
            } catch {
              data.value.settings = previous
            }
          }
        }
        const wire = toIpcError(error)
        toast.error(t('toasts.settingsError'), wire.message, wire.detail)
        return false
      }
    })

    settingsQueue = task.then(
      () => true,
      () => false,
    )
    return task
  }

  async function updateWorkspaceKeyPrefix(prefix: string): Promise<boolean> {
    try {
      const workspace = await api.updateWorkspaceKeyPrefix(prefix)
      if (data.value) data.value.workspace = workspace
      toast.success(t('settings.workspace.keyPrefixSaved'))
      return true
    } catch (error) {
      const wire = toIpcError(error)
      toast.error(t('settings.workspace.keyPrefixError'), wire.message, wire.detail)
      return false
    }
  }

  async function setTheme(theme: ThemePreference): Promise<boolean> {
    return saveSettings({ theme })
  }

  async function setLocale(locale: LocalePreference): Promise<boolean> {
    return saveSettings({ locale })
  }

  async function setSidebarCollapsed(collapsed: boolean): Promise<boolean> {
    return saveSettings({ sidebarCollapsed: collapsed })
  }

  async function setAutoBackupEnabled(enabled: boolean): Promise<boolean> {
    return saveSettings({ autoBackupEnabled: enabled })
  }

  async function setBackupRetention(retention: number): Promise<boolean> {
    return saveSettings({ backupRetention: retention })
  }

  let routeTimer: ReturnType<typeof setTimeout> | null = null
  function rememberRoute(path: string): void {
    if (!settings.value || settings.value.lastRoute === path) {
      return
    }
    if (routeTimer) {
      clearTimeout(routeTimer)
    }
    routeTimer = setTimeout(() => {
      routeTimer = null
      void saveSettings({ lastRoute: path })
    }, 400)
  }

  async function createWorkspace(name: string): Promise<Workspace> {
    const created = await api.createWorkspace(name)
    if (data.value) {
      data.value.workspace = created
    }
    return created
  }

  async function completeOnboarding(): Promise<boolean> {
    try {
      const saved = await api.completeOnboarding()
      if (data.value) {
        data.value.settings = saved
      }
      return true
    } catch (error) {
      const wire = toIpcError(error)
      toast.error(t('onboarding.finishError'), wire.message, wire.detail)
      return false
    }
  }

  async function refreshBackups(): Promise<BackupInfo[]> {
    try {
      const list = await api.listBackups()
      if (data.value) {
        data.value.backups = list
      }
      return list
    } catch (error) {
      const wire = toIpcError(error)
      toast.error(t('settings.storage.listError'), wire.message, wire.detail)
      return []
    }
  }

  async function createBackup(): Promise<BackupInfo | null> {
    try {
      const created = await api.createBackup()
      await refreshBackups()
      toast.success(
        t('settings.storage.createSuccess'),
        t('settings.storage.createSuccessBody', { id: created.id }),
      )
      return created
    } catch (error) {
      const wire = toIpcError(error)
      toast.error(t('settings.storage.createError'), wire.message, wire.detail)
      return null
    }
  }

  async function restoreBackup(id: string): Promise<RestoreResult | null> {
    try {
      const result = await api.restoreBackup(id)
      data.value = result.bootstrap
      toast.success(
        t('settings.storage.restoreSuccess'),
        t('settings.storage.restoreSuccessBody', {
          id: result.safetyBackup.id,
        }),
      )
      return result
    } catch (error) {
      const wire = toIpcError(error)
      toast.error(t('settings.storage.restoreError'), wire.message, wire.detail)
      return null
    }
  }

  return {
    data,
    loading,
    startupError,
    lastSavedAt,
    settings,
    workspace,
    backups,
    appVersion,
    schemaVersion,
    startupWarning,
    resolvedTheme,
    resolvedLocale,
    sidebarCollapsed,
    needsOnboarding,
    systemPrefersDark,
    load,
    saveSettings,
    setTheme,
    updateWorkspaceKeyPrefix,
    setLocale,
    setSidebarCollapsed,
    setAutoBackupEnabled,
    setBackupRetention,
    rememberRoute,
    createWorkspace,
    completeOnboarding,
    refreshBackups,
    createBackup,
    restoreBackup,
  }
})
