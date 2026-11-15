import { invoke, isTauri } from '@tauri-apps/api/core'

import type {
  AppSettings,
  BackupInfo,
  Bootstrap,
  CommandName,
  IpcError,
  RestoreResult,
  SettingsPatch,
  Workspace,
} from '@/types/ipc'

export type Transport = <T>(
  command: CommandName,
  args?: Record<string, unknown>,
) => Promise<T>

let devTransport: Transport | null = null

/**
 * Test/visual-gate bridge. It only exists when explicitly enabled at build time
 * (`VITE_VISUAL_HARNESS=true`) or during local development. Production builds
 * cannot install one.
 */
export function setDevTransport(next: Transport): void {
  if (!import.meta.env.DEV && import.meta.env.VITE_VISUAL_HARNESS !== 'true') {
    throw new Error('A development IPC transport is not allowed in this build.')
  }
  devTransport = next
}

export function toIpcError(error: unknown): IpcError {
  if (typeof error === 'object' && error !== null) {
    const candidate = error as Partial<IpcError>
    if (typeof candidate.message === 'string' && candidate.message.length > 0) {
      return {
        code:
          typeof candidate.code === 'string' && candidate.code.length > 0
            ? candidate.code
            : 'internal',
        message: candidate.message,
        field: typeof candidate.field === 'string' ? candidate.field : undefined,
        detail:
          typeof candidate.detail === 'string' ? candidate.detail : undefined,
      }
    }
  }
  if (error instanceof Error) {
    return { code: 'internal', message: error.message }
  }
  return { code: 'internal', message: String(error) }
}

export function isIpcError(value: unknown): value is IpcError {
  return (
    typeof value === 'object' &&
    value !== null &&
    typeof (value as IpcError).code === 'string' &&
    typeof (value as IpcError).message === 'string'
  )
}

async function call<T>(
  command: CommandName,
  args?: Record<string, unknown>,
): Promise<T> {
  if (devTransport) {
    return devTransport<T>(command, args)
  }
  if (!isTauri()) {
    throw {
      code: 'desktop_unavailable',
      message:
        'The desktop bridge is not available. Start ForgeDesk with "npm run tauri dev".',
    } satisfies IpcError
  }
  try {
    return await invoke<T>(command, args)
  } catch (error) {
    throw toIpcError(error)
  }
}

export const api = {
  bootstrap: () => call<Bootstrap>('bootstrap'),
  createWorkspace: (name: string) =>
    call<Workspace>('create_workspace', { name }),
  saveSettings: (patch: SettingsPatch) =>
    call<AppSettings>('save_settings', { patch }),
  completeOnboarding: () => call<AppSettings>('complete_onboarding'),
  saveWindowState: (window: AppSettings['window']) =>
    call<AppSettings>('save_window_state', { window }),
  listBackups: () => call<BackupInfo[]>('list_backups'),
  createBackup: () => call<BackupInfo>('create_backup'),
  restoreBackup: (id: string) => call<RestoreResult>('restore_backup', { id }),
}
