import { createPinia, setActivePinia } from 'pinia'
import { nextTick } from 'vue'
import { beforeEach, describe, expect, it } from 'vitest'

import { setDevTransport, type Transport } from '@/lib/ipc'
import type {
  AppSettings,
  BackupInfo,
  Bootstrap,
  CommandName,
  SettingsPatch,
  Workspace,
} from '@/types/ipc'
import { useAppStore } from './app'

const timestamp = '2026-11-15T12:00:00.000Z'

function defaultSettings(): AppSettings {
  return {
    locale: 'en',
    theme: 'dark',
    sidebarCollapsed: false,
    lastRoute: '/my-issues',
    onboardingCompleted: false,
    detailPaneOpen: false,
    autoBackupEnabled: true,
    backupRetention: 20,
    window: { width: 1440, height: 900, x: null, y: null, maximized: false },
  }
}

function backup(id = 'forgedesk-test'): BackupInfo {
  return {
    id,
    filePath: `/tmp/${id}.db`,
    createdAt: timestamp,
    schemaVersion: 1,
    sizeBytes: 24_576,
    appVersion: '0.1.0',
    valid: true,
  }
}

let workspace: Workspace | null
let settings: AppSettings
let backups: BackupInfo[]
let failSettings = false
let transport: Transport

function bootstrap(): Bootstrap {
  return structuredClone({
    appVersion: '0.1.0',
    schemaVersion: 1,
    dataDir: '/tmp/forgedesk',
    dbPath: '/tmp/forgedesk/forgedesk.db',
    backupsDir: '/tmp/forgedesk/backups',
    workspace,
    settings,
    backups,
    startupWarning: null,
  })
}

function installTransport(): void {
  transport = async <T,>(command: CommandName, args?: Record<string, unknown>) => {
    switch (command) {
      case 'bootstrap':
        return bootstrap() as T
      case 'create_workspace':
        workspace = {
          id: 'ws-1',
          name: String(args?.name),
          keyPrefix: 'FD',
          createdAt: timestamp,
          updatedAt: timestamp,
        }
        return structuredClone(workspace) as T
      case 'save_settings':
        if (failSettings) {
          throw { code: 'validation', message: 'Settings rejected.' }
        }
        settings = { ...settings, ...(args?.patch as SettingsPatch) }
        return structuredClone(settings) as T
      case 'complete_onboarding':
        settings = { ...settings, onboardingCompleted: true }
        return structuredClone(settings) as T
      case 'save_window_state':
        return structuredClone(settings) as T
      case 'list_backups':
        return structuredClone(backups) as T
      case 'create_backup': {
        const created = backup(`forgedesk-${backups.length + 1}`)
        backups = [created, ...backups]
        return structuredClone(created) as T
      }
      case 'restore_backup': {
        const restored = backups.find((item) => item.id === args?.id)
        if (!restored) {
          throw { code: 'not_found', message: 'Not found' }
        }
        settings = { ...settings, theme: 'dark' }
        const safetyBackup = backup('forgedesk-safety')
        backups = [safetyBackup, ...backups]
        return structuredClone({
          restored,
          safetyBackup,
          bootstrap: bootstrap(),
        }) as T
      }
      default:
        throw { code: 'unsupported_capability', message: String(command) }
    }
  }
  setDevTransport(transport)
}

describe('app store', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    workspace = null
    settings = defaultSettings()
    backups = []
    failSettings = false
    installTransport()
  })

  it('loads an empty installation and exposes onboarding', async () => {
    const app = useAppStore()
    await app.load()
    expect(app.loading).toBe(false)
    expect(app.startupError).toBeNull()
    expect(app.workspace).toBeNull()
    expect(app.needsOnboarding).toBe(true)
    expect(app.resolvedTheme).toBe('dark')
  })

  it('creates a workspace, completes onboarding and applies appearance', async () => {
    const app = useAppStore()
    await app.load()

    const created = await app.createWorkspace('Product Studio')
    expect(created.name).toBe('Product Studio')
    expect(app.workspace?.name).toBe('Product Studio')
    expect(app.needsOnboarding).toBe(true)

    const completed = await app.completeOnboarding()
    expect(completed).toBe(true)
    expect(app.needsOnboarding).toBe(false)

    const saved = await app.saveSettings({ theme: 'light', locale: 'ru' })
    expect(saved).toBe(true)
    expect(app.settings?.theme).toBe('light')
    expect(app.settings?.locale).toBe('ru')
    expect(app.resolvedTheme).toBe('light')
    expect(document.documentElement.dataset.theme).toBe('light')
    expect(document.documentElement.lang).toBe('ru')
  })

  it('serializes overlapping settings writes', async () => {
    let saves = 0
    setDevTransport(async <T,>(command: CommandName, args?: Record<string, unknown>) => {
      if (command === 'bootstrap') {
        return bootstrap() as T
      }
      if (command === 'save_settings') {
        saves += 1
        const patch = args?.patch as SettingsPatch
        await new Promise((resolve) => setTimeout(resolve, saves === 1 ? 25 : 0))
        settings = { ...settings, ...patch }
        return structuredClone(settings) as T
      }
      throw { code: 'unsupported_capability', message: String(command) }
    })

    const app = useAppStore()
    await app.load()
    const first = app.saveSettings({ theme: 'light' })
    const second = app.saveSettings({ locale: 'ru' })
    await Promise.all([first, second])

    expect(saves).toBe(2)
    expect(settings.theme).toBe('light')
    expect(settings.locale).toBe('ru')
    expect(app.settings?.theme).toBe('light')
    expect(app.settings?.locale).toBe('ru')
  })

  it('rolls back optimistic settings when persistence fails', async () => {
    const app = useAppStore()
    await app.load()
    failSettings = true

    const saved = await app.saveSettings({ theme: 'light' })
    expect(saved).toBe(false)
    expect(app.settings?.theme).toBe('dark')
    expect(app.resolvedTheme).toBe('dark')
  })

  it('creates and restores backups', async () => {
    const app = useAppStore()
    await app.load()

    const created = await app.createBackup()
    expect(created).not.toBeNull()
    expect(app.backups).toHaveLength(1)

    await app.saveSettings({ theme: 'light' })
    expect(app.resolvedTheme).toBe('light')

    const result = await app.restoreBackup(created!.id)
    expect(result).not.toBeNull()
    expect(app.settings?.theme).toBe('dark')
    expect(app.resolvedTheme).toBe('dark')
    await nextTick()
    expect(document.documentElement.dataset.theme).toBe('dark')
    expect(app.backups.some((item) => item.id === 'forgedesk-safety')).toBe(true)
  })

  it('reports startup errors without dropping data', async () => {
    setDevTransport(async () => {
      throw {
        code: 'local_db',
        message: 'The local database could not be opened.',
      }
    })
    const app = useAppStore()
    await app.load()
    expect(app.loading).toBe(false)
    expect(app.data).toBeNull()
    expect(app.startupError?.code).toBe('local_db')
  })
})
