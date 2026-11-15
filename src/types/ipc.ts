export type ThemePreference = 'dark' | 'light' | 'system'
export type LocalePreference = 'system' | 'en' | 'ru'
export type ResolvedTheme = 'dark' | 'light'
export type ResolvedLocale = 'en' | 'ru'

export interface WindowState {
  width: number
  height: number
  x: number | null
  y: number | null
  maximized: boolean
}

export interface AppSettings {
  locale: LocalePreference
  theme: ThemePreference
  sidebarCollapsed: boolean
  lastRoute: string
  onboardingCompleted: boolean
  detailPaneOpen: boolean
  autoBackupEnabled: boolean
  backupRetention: number
  window: WindowState
}

export interface Workspace {
  id: string
  name: string
  keyPrefix: string
  createdAt: string
  updatedAt: string
}

export interface BackupInfo {
  id: string
  filePath: string
  createdAt: string
  schemaVersion: number
  sizeBytes: number
  appVersion: string
  valid: boolean
}

export interface Bootstrap {
  appVersion: string
  schemaVersion: number
  dataDir: string
  dbPath: string
  backupsDir: string
  workspace: Workspace | null
  settings: AppSettings
  backups: BackupInfo[]
  startupWarning: string | null
}

export interface RestoreResult {
  restored: BackupInfo
  safetyBackup: BackupInfo
  bootstrap: Bootstrap
}

export interface SettingsPatch {
  locale?: LocalePreference
  theme?: ThemePreference
  sidebarCollapsed?: boolean
  lastRoute?: string
  onboardingCompleted?: boolean
  detailPaneOpen?: boolean
  autoBackupEnabled?: boolean
  backupRetention?: number
  window?: WindowState
}

export interface IpcError {
  code: string
  message: string
  field?: string
  detail?: string
}

export type CommandName =
  | 'bootstrap'
  | 'create_workspace'
  | 'save_settings'
  | 'complete_onboarding'
  | 'save_window_state'
  | 'list_backups'
  | 'create_backup'
  | 'restore_backup'
