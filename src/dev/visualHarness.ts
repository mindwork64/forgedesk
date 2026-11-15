/**
 * Visual-gate harness.
 *
 * This module is loaded only when the frontend is built with
 * `VITE_VISUAL_HARNESS=true` (or during Vite dev). It provides deterministic
 * in-memory responses so UI Gate screenshots can be captured in a headless
 * browser when the native WebKit shell is unavailable. It is not part of the
 * production bundle and never claims to be a real integration.
 */
import { setDevTransport } from '@/lib/ipc'
import { handleIssueFixtureCommand } from './issueFixtures'
import { handlePlanningFixtureCommand } from './planningFixtures'
import { handleStage4FixtureCommand } from './stage4Fixtures'
import type {
  AppSettings,
  BackupInfo,
  Bootstrap,
  IpcError,
  RestoreResult,
  SettingsPatch,
  CommandName,
  Workspace,
} from '@/types/ipc'

const params = new URLSearchParams(window.location.search)

function isoOffset(hoursAgo: number): string {
  return new Date(Date.now() - hoursAgo * 3_600_000).toISOString()
}

const dataDir = '/home/dev/.local/share/dev.forgedesk.app'

let settings: AppSettings = {
  locale: params.get('locale') === 'ru' ? 'ru' : 'en',
  theme: params.get('theme') === 'light' ? 'light' : 'dark',
  sidebarCollapsed: params.get('sidebar') === 'collapsed',
  lastRoute: params.get('route') ?? '/my-issues',
  onboardingCompleted: params.get('onboarding') !== '1',
  detailPaneOpen: false,
  autoBackupEnabled: true,
  backupRetention: 20,
  window: {
    width: 1440,
    height: 900,
    x: null,
    y: null,
    maximized: false,
  },
}

let workspace: Workspace | null =
  params.get('onboarding') === '1'
    ? null
    : {
        id: '0198f0aa-7c3e-7a2b-9f10-2c4c8f6d1a01',
        name: 'My Workspace',
        keyPrefix: 'FD',
        createdAt: isoOffset(720),
        updatedAt: isoOffset(2),
      }

let backups: BackupInfo[] = [
  {
    id: 'forgedesk-20261115-043012-a1b2c3d4',
    filePath: `${dataDir}/backups/forgedesk-20261115-043012-a1b2c3d4.db`,
    createdAt: isoOffset(2),
    schemaVersion: 5,
    sizeBytes: 28_672,
    appVersion: '0.1.0',
    valid: true,
  },
  {
    id: 'forgedesk-20261114-102211-9e8d7c6b',
    filePath: `${dataDir}/backups/forgedesk-20261114-102211-9e8d7c6b.db`,
    createdAt: isoOffset(20),
    schemaVersion: 5,
    sizeBytes: 28_672,
    appVersion: '0.1.0',
    valid: true,
  },
  {
    id: 'forgedesk-20261113-090403-5f4e3d2c',
    filePath: `${dataDir}/backups/forgedesk-20261113-090403-5f4e3d2c.db`,
    createdAt: isoOffset(44),
    schemaVersion: 5,
    sizeBytes: 24_576,
    appVersion: '0.1.0',
    valid: true,
  },
]

function bootstrap(): Bootstrap {
  if (params.get('error') === '1') {
    fail('local_db', 'The local database could not be opened.')
  }
  return structuredClone({
    appVersion: '0.1.0',
    schemaVersion: 5,
    dataDir,
    dbPath: `${dataDir}/forgedesk.db`,
    backupsDir: `${dataDir}/backups`,
    workspace,
    settings,
    backups,
    startupWarning: null,
  })
}

function mergeSettings(patch: SettingsPatch): AppSettings {
  settings = {
    ...settings,
    ...patch,
    window: { ...settings.window, ...(patch.window ?? {}) },
  }
  return structuredClone(settings)
}

function fail(code: string, message: string): never {
  const error: IpcError = { code, message }
  throw error
}

export function installVisualHarness(): void {
  console.warn(
    '[ForgeDesk] Visual harness enabled. Responses come from in-memory fixtures, not SQLite.',
  )

  setDevTransport(async <T,>(command: CommandName, args?: Record<string, unknown>): Promise<T> => {
    await new Promise((resolve) => setTimeout(resolve, 15))

    const stage4Result = handleStage4FixtureCommand<T>(command, args)
    if (stage4Result.handled) {
      return stage4Result.value as T
    }

    const planningResult = handlePlanningFixtureCommand<T>(command, args)
    if (planningResult.handled) {
      return planningResult.value as T
    }

    const issueResult = handleIssueFixtureCommand<T>(command, args)
    if (issueResult.handled) {
      return issueResult.value as T
    }

    switch (command) {
      case 'bootstrap':
        return bootstrap() as T
      case 'create_workspace': {
        if (workspace) {
          fail('conflict', 'A workspace already exists in this installation.')
        }
        const name = String(args?.name ?? 'My Workspace')
        workspace = {
          id: crypto.randomUUID(),
          name,
          keyPrefix: 'FD',
          createdAt: isoOffset(0),
          updatedAt: isoOffset(0),
        }
        return structuredClone(workspace) as T
      }
      case 'save_settings':
        return mergeSettings((args?.patch ?? {}) as SettingsPatch) as T
      case 'complete_onboarding':
        settings = { ...settings, onboardingCompleted: true }
        return structuredClone(settings) as T
      case 'save_window_state':
        return mergeSettings({
          window: args?.window as AppSettings['window'],
        }) as T
      case 'list_backups':
        return structuredClone(backups) as T
      case 'create_backup': {
        const created: BackupInfo = {
          id: `forgedesk-${Date.now()}-${Math.random().toString(16).slice(2, 10)}`,
          filePath: `${dataDir}/backups/new.db`,
          createdAt: isoOffset(0),
          schemaVersion: 5,
          sizeBytes: 28_672,
          appVersion: '0.1.0',
          valid: true,
        }
        backups = [created, ...backups]
        return structuredClone(created) as T
      }
      case 'restore_backup': {
        const id = String(args?.id ?? '')
        const restored = backups.find((item) => item.id === id)
        if (!restored) {
          fail('not_found', 'Backup was not found.')
        }
        const safetyBackup: BackupInfo = {
          id: `forgedesk-safety-${Date.now()}`,
          filePath: `${dataDir}/backups/safety.db`,
          createdAt: isoOffset(0),
          schemaVersion: 5,
          sizeBytes: 28_672,
          appVersion: '0.1.0',
          valid: true,
        }
        backups = [safetyBackup, ...backups]
        const result: RestoreResult = {
          restored,
          safetyBackup,
          bootstrap: bootstrap(),
        }
        return structuredClone(result) as T
      }
      default:
        return fail('unsupported_capability', `Harness does not implement ${command}.`)
    }
  })
}
