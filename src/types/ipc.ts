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
  | 'list_workflow'
  | 'list_issue_statuses'
  | 'list_labels'
  | 'list_projects'
  | 'list_issues'
  | 'get_issue'
  | 'create_issue'
  | 'update_issue'
  | 'archive_issue'
  | 'batch_update_issues'
  | 'duplicate_issue'
  | 'add_issue_comment'
  | 'add_issue_relation'
  | 'remove_issue_relation'

export type Priority = 'none' | 'low' | 'medium' | 'high' | 'urgent'
export type IssueType =
  | 'bug'
  | 'feature'
  | 'task'
  | 'improvement'
  | 'chore'
  | 'research'
export type StatusCategory =
  | 'backlog'
  | 'unstarted'
  | 'started'
  | 'completed'
  | 'canceled'
export type StatusColor =
  | 'neutral'
  | 'info'
  | 'accent'
  | 'success'
  | 'warning'
  | 'danger'
export type IssueSource = 'local' | 'github' | 'gitlab' | 'gitea'

export interface StatusSummary {
  id: string
  name: string
  category: StatusCategory
  color: StatusColor
  position: number
  systemKey: string | null
}

export interface WorkflowStatus extends StatusSummary {
  isDefault: boolean
}

export interface Workflow {
  id: string
  name: string
  isDefault: boolean
  statuses: WorkflowStatus[]
}

export interface Label {
  id: string
  name: string
  color: StatusColor
}

export interface ProjectRef {
  id: string
  name: string
}

export interface IssueSummary {
  id: string
  key: string
  keyNumber: number
  title: string
  status: StatusSummary
  priority: Priority
  issueType: IssueType
  project: ProjectRef | null
  labels: Label[]
  estimate: number | null
  startDate: string | null
  dueDate: string | null
  parentId: string | null
  source: IssueSource
  remoteUrl: string | null
  archivedAt: string | null
  createdAt: string
  updatedAt: string
  completedAt: string | null
  subIssueCount: number
}

export type CommentAuthor = 'local' | 'remote'

export interface Comment {
  id: string
  issueId: string
  body: string
  author: CommentAuthor
  createdAt: string
  updatedAt: string
}

export interface ActivityEvent {
  id: string
  issueId: string
  kind: string
  data: unknown
  createdAt: string
}

export type RelationKind =
  | 'blocks'
  | 'blockedBy'
  | 'related'
  | 'duplicate'
  | 'duplicatedBy'
export type RelationInputKind = 'blocks' | 'related' | 'duplicate'

export interface RelatedIssue {
  id: string
  key: string
  title: string
  status: StatusSummary
}

export interface IssueRelation {
  id: string
  kind: RelationKind
  issue: RelatedIssue
}

export interface IssueDetail {
  summary: IssueSummary
  description: string
  parent: IssueSummary | null
  children: IssueSummary[]
  relations: IssueRelation[]
  comments: Comment[]
  activity: ActivityEvent[]
}

export type LogicalOp = 'and' | 'or'
export type FilterField =
  | 'status'
  | 'statusCategory'
  | 'priority'
  | 'type'
  | 'project'
  | 'label'
  | 'estimate'
  | 'startDate'
  | 'dueDate'
  | 'createdAt'
  | 'updatedAt'
  | 'title'
  | 'parent'
  | 'archived'
  | 'source'
export type FilterOperator =
  | 'eq'
  | 'neq'
  | 'in'
  | 'notIn'
  | 'contains'
  | 'isEmpty'
  | 'isNotEmpty'
  | 'gt'
  | 'gte'
  | 'lt'
  | 'lte'

export type FilterNode =
  | {
      kind: 'condition'
      field: FilterField
      operator: FilterOperator
      value?: unknown
    }
  | {
      kind: 'group'
      op: LogicalOp
      conditions: FilterNode[]
    }

export type SortField =
  | 'updatedAt'
  | 'createdAt'
  | 'priority'
  | 'status'
  | 'title'
  | 'dueDate'
  | 'estimate'
export type SortDirection = 'asc' | 'desc'

export interface SortSpec {
  field: SortField
  direction: SortDirection
}

export interface IssueQuery {
  filter?: FilterNode | null
  sort?: SortSpec[]
  search?: string | null
  includeArchived?: boolean
  limit?: number | null
  offset?: number | null
}

export interface CreateIssueInput {
  title: string
  description?: string
  statusId?: string | null
  priority?: Priority
  issueType?: IssueType
  estimate?: number | null
  startDate?: string | null
  dueDate?: string | null
  projectId?: string | null
  parentId?: string | null
  labelNames?: string[]
}

export interface UpdateIssueInput {
  title?: string | null
  description?: string | null
  statusId?: string | null
  priority?: Priority | null
  issueType?: IssueType | null
  estimate?: number | null
  startDate?: string | null
  dueDate?: string | null
  projectId?: string | null
  parentId?: string | null
  labelNames?: string[] | null
  clearFields?: string[]
}

export interface BatchIssueInput {
  statusId?: string | null
  priority?: Priority | null
  issueType?: IssueType | null
  addLabelNames?: string[]
  removeLabelNames?: string[]
  archive?: boolean | null
}
