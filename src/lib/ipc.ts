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
  SavedView,
  SavedViewInput,
  Cycle,
  CycleDetail,
  CycleInput,
  Document,
  DocumentInput,
  Initiative,
  InitiativeDetail,
  InitiativeInput,
  Milestone,
  MilestoneInput,
  Project,
  ProjectDetail,
  ProjectInput,
  ProjectQuery,
  ApiControl,
  ApiMonitorState,
  GitAvailability,
  GitDiffStat,
  GitLogEntry,
  GitOverview,
  Repository,
  RepositoryInput,
  DeviceFlowStart,
  DeviceFlowPoll,
  GitHubConnection,
  GitHubRepo,
  GitHubImportResult,
  GitHubPull,
  PullRequestDetail,
  GitHubSearchIssue,
  CiOverview,
  GitHubCheckRun,
  PullLink,
  PullCapabilities,
  GitHubMergeResult,
  GitHubReview,
  InboxEvent,
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
  listSavedViews: () => call<SavedView[]>('list_saved_views'),
  createSavedView: (input: SavedViewInput) =>
    call<SavedView>('create_saved_view', { input }),
  updateSavedView: (id: string, input: SavedViewInput) =>
    call<SavedView>('update_saved_view', { id, input }),
  deleteSavedView: (id: string) => call<void>('delete_saved_view', { id }),
  listProjectCards: (query: ProjectQuery = {}) =>
    call<Project[]>('list_project_cards', { query }),
  getProject: (id: string) => call<ProjectDetail>('get_project', { id }),
  createProject: (input: ProjectInput) =>
    call<Project>('create_project', { input }),
  updateProject: (id: string, input: ProjectInput) =>
    call<Project>('update_project', { id, input }),
  archiveProject: (id: string, archived: boolean) =>
    call<Project>('archive_project', { id, archived }),
  createMilestone: (projectId: string, input: MilestoneInput) =>
    call<Milestone>('create_milestone', { projectId, input }),
  updateMilestone: (id: string, input: MilestoneInput) =>
    call<Milestone>('update_milestone', { id, input }),
  assignMilestoneIssue: (
    milestoneId: string,
    issueId: string,
    assigned: boolean,
  ) =>
    call<void>('assign_milestone_issue', { milestoneId, issueId, assigned }),
  listInitiatives: () => call<Initiative[]>('list_initiatives'),
  getInitiative: (id: string) => call<InitiativeDetail>('get_initiative', { id }),
  createInitiative: (input: InitiativeInput) =>
    call<Initiative>('create_initiative', { input }),
  updateInitiative: (id: string, input: InitiativeInput) =>
    call<Initiative>('update_initiative', { id, input }),
  listCycles: () => call<Cycle[]>('list_cycles'),
  getCycle: (id: string) => call<CycleDetail>('get_cycle', { id }),
  createCycle: (input: CycleInput) => call<Cycle>('create_cycle', { input }),
  updateCycle: (id: string, input: CycleInput) =>
    call<Cycle>('update_cycle', { id, input }),
  assignCycleIssue: (cycleId: string, issueId: string, assigned: boolean) =>
    call<void>('assign_cycle_issue', { cycleId, issueId, assigned }),
  listDocuments: () => call<Document[]>('list_documents'),
  getDocument: (id: string) => call<Document>('get_document', { id }),
  createDocument: (input: DocumentInput) =>
    call<Document>('create_document', { input }),
  updateDocument: (id: string, input: DocumentInput) =>
    call<Document>('update_document', { id, input }),
  deleteDocument: (id: string) => call<void>('delete_document', { id }),
  exportDocument: (id: string) => call<string>('export_document', { id }),
  gitAvailable: () => call<GitAvailability>('git_available'),
  listRepositories: () => call<Repository[]>('list_repositories'),
  addLocalRepository: (input: RepositoryInput) =>
    call<Repository>('add_local_repository', { input }),
  removeRepository: (id: string) => call<void>('remove_repository', { id }),
  getGitOverview: (id: string) => call<GitOverview>('get_git_overview', { id }),
  createGitBranch: (id: string, branch: string) =>
    call<GitOverview>('create_git_branch', { id, branch }),
  switchGitBranch: (id: string, branch: string) =>
    call<GitOverview>('switch_git_branch', { id, branch }),
  getGitLog: (id: string, limit = 12) =>
    call<GitLogEntry[]>('get_git_log', { id, limit }),
  getGitDiff: (id: string) => call<GitDiffStat>('get_git_diff', { id }),
  getApiMonitor: () => call<ApiMonitorState>('get_api_monitor'),
  setApiPaused: (paused: boolean, reason = '') =>
    call<ApiControl>('set_api_paused', { paused, reason }),
  exportApiDiagnostics: () => call<string>('export_api_diagnostics'),
  githubStartDeviceFlow: () =>
    call<DeviceFlowStart>('github_start_device_flow'),
  githubPollDeviceFlow: () => call<DeviceFlowPoll>('github_poll_device_flow'),
  githubConnection: () => call<GitHubConnection | null>('github_connection'),
  githubRefresh: () => call<GitHubConnection>('github_refresh'),
  githubDisconnect: () => call<void>('github_disconnect'),
  githubDiscoverRepositories: () =>
    call<GitHubRepo[]>('github_discover_repositories'),
  githubImportIssues: (owner: string, repo: string) =>
    call<GitHubImportResult>('github_import_issues', { owner, repo }),
  githubListPulls: (owner: string, repo: string, pullState: string) =>
    call<GitHubPull[]>('github_list_pulls', { owner, repo, pullState }),
  githubPullDetail: (owner: string, repo: string, number: number) =>
    call<PullRequestDetail>('github_pull_detail', { owner, repo, number }),
  githubReviewRequested: (owner: string, repo: string) =>
    call<GitHubSearchIssue[]>('github_review_requested', { owner, repo }),
  githubCiOverview: (owner: string, repo: string) =>
    call<CiOverview>('github_ci_overview', { owner, repo }),
  githubCommitChecks: (owner: string, repo: string, reference: string) =>
    call<GitHubCheckRun[]>('github_commit_checks', { owner, repo, reference }),
  githubPullLink: (owner: string, repo: string, number: number) =>
    call<PullLink | null>('github_pull_link', { owner, repo, number }),
  githubLinkPull: (owner: string, repo: string, number: number, issueKey: string) =>
    call<PullLink>('github_link_pull', { owner, repo, number, issueKey }),
  githubUnlinkPull: (owner: string, repo: string, number: number) =>
    call<void>('github_unlink_pull', { owner, repo, number }),
  githubPullCapabilities: (owner: string, repo: string) =>
    call<PullCapabilities>('github_pull_capabilities', { owner, repo }),
  githubMergePull: (
    owner: string,
    repo: string,
    number: number,
    method: string,
    expectedSha: string | null,
    confirmed: boolean,
  ) =>
    call<GitHubMergeResult>('github_merge_pull', {
      owner,
      repo,
      number,
      method,
      expectedSha,
      confirmed,
    }),
  githubClosePull: (
    owner: string,
    repo: string,
    number: number,
    confirmed: boolean,
  ) => call<GitHubPull>('github_close_pull', { owner, repo, number, confirmed }),
  githubSubmitReview: (
    owner: string,
    repo: string,
    number: number,
    event: string,
    body: string,
    confirmed: boolean,
  ) =>
    call<GitHubReview>('github_submit_review', {
      owner,
      repo,
      number,
      event,
      body,
      confirmed,
    }),
  inboxList: (filter: string) => call<InboxEvent[]>('inbox_list', { filter }),
  inboxMarkRead: (eventId: string, read: boolean) =>
    call<void>('inbox_mark_read', { eventId, read }),
  inboxArchive: (eventId: string) =>
    call<void>('inbox_archive', { eventId }),
  inboxMarkAllRead: () => call<number>('inbox_mark_all_read'),
}
