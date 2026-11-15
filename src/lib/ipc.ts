import { invoke, isTauri } from '@tauri-apps/api/core'

import type {
  AppSettings,
  BackupInfo,
  BatchIssueInput,
  Bootstrap,
  CommandName,
  CreateIssueInput,
  IssueDetail,
  IssueQuery,
  IssueSummary,
  IpcError,
  Label,
  ProjectRef,
  RelationInputKind,
  RestoreResult,
  SettingsPatch,
  StatusSummary,
  UpdateIssueInput,
  Workflow,
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
  listWorkflow: () => call<Workflow>('list_workflow'),
  listIssueStatuses: () => call<StatusSummary[]>('list_issue_statuses'),
  listLabels: () => call<Label[]>('list_labels'),
  listProjects: () => call<ProjectRef[]>('list_projects'),
  listIssues: (query: IssueQuery) => call<IssueSummary[]>('list_issues', { query }),
  getIssue: (id: string) => call<IssueDetail>('get_issue', { id }),
  createIssue: (input: CreateIssueInput) =>
    call<IssueDetail>('create_issue', { input }),
  updateIssue: (id: string, input: UpdateIssueInput) =>
    call<IssueDetail>('update_issue', { id, input }),
  archiveIssue: (id: string, archived: boolean) =>
    call<IssueDetail>('archive_issue', { id, archived }),
  batchUpdateIssues: (ids: string[], input: BatchIssueInput) =>
    call<IssueDetail[]>('batch_update_issues', { ids, input }),
  duplicateIssue: (id: string) => call<IssueDetail>('duplicate_issue', { id }),
  addIssueComment: (issueId: string, body: string) =>
    call<IssueDetail>('add_issue_comment', { issueId, body }),
  addIssueRelation: (
    issueId: string,
    relatedIssueId: string,
    kind: RelationInputKind,
  ) =>
    call<IssueDetail>('add_issue_relation', {
      issueId,
      relatedIssueId,
      kind,
    }),
  removeIssueRelation: (issueId: string, relationId: string) =>
    call<IssueDetail>('remove_issue_relation', { issueId, relationId }),
}
