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
  | 'list_saved_views'
  | 'create_saved_view'
  | 'update_saved_view'
  | 'delete_saved_view'
  | 'list_project_cards'
  | 'get_project'
  | 'create_project'
  | 'update_project'
  | 'archive_project'
  | 'create_milestone'
  | 'update_milestone'
  | 'assign_milestone_issue'
  | 'list_initiatives'
  | 'get_initiative'
  | 'create_initiative'
  | 'update_initiative'
  | 'list_cycles'
  | 'get_cycle'
  | 'create_cycle'
  | 'update_cycle'
  | 'assign_cycle_issue'
  | 'list_documents'
  | 'get_document'
  | 'create_document'
  | 'update_document'
  | 'delete_document'
  | 'export_document'
  | 'git_available'
  | 'list_repositories'
  | 'add_local_repository'
  | 'remove_repository'
  | 'get_git_overview'
  | 'create_git_branch'
  | 'switch_git_branch'
  | 'get_git_log'
  | 'get_git_diff'
  | 'get_api_monitor'
  | 'set_api_paused'
  | 'export_api_diagnostics'
  | 'github_start_device_flow'
  | 'github_poll_device_flow'
  | 'github_connection'
  | 'github_refresh'
  | 'github_disconnect'
  | 'github_discover_repositories'
  | 'github_import_issues'
  | 'github_list_pulls'
  | 'github_pull_detail'
  | 'github_review_requested'
  | 'github_ci_overview'
  | 'github_commit_checks'
  | 'github_pull_link'
  | 'github_link_pull'
  | 'github_unlink_pull'
  | 'github_pull_capabilities'
  | 'github_merge_pull'
  | 'github_close_pull'
  | 'github_submit_review'
  | 'inbox_list'
  | 'inbox_mark_read'
  | 'inbox_archive'
  | 'inbox_mark_all_read'
  | 'get_insights'

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

export interface FilterCondition {
  kind: 'condition'
  field: FilterField
  operator: FilterOperator
  value?: unknown
}

export interface FilterGroup {
  kind: 'group'
  op: LogicalOp
  conditions: FilterNode[]
}

export type FilterNode = FilterCondition | FilterGroup

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

export interface SavedView {
  id: string
  name: string
  filters: FilterNode | null
  sort: SortSpec[]
  groupBy: string | null
  columns: string[]
  density: 'standard' | 'compact'
  isPinned: boolean
  position: number
  createdAt: string
  updatedAt: string
}

export interface SavedViewInput {
  name: string
  filters?: FilterNode | null
  sort?: SortSpec[]
  groupBy?: string | null
  columns?: string[]
  density?: 'standard' | 'compact' | null
  isPinned?: boolean | null
}

export type PlanningStatus = 'planned' | 'active' | 'paused' | 'completed' | 'canceled'
export type MilestoneStatus = 'planned' | 'active' | 'completed' | 'canceled'
export type CycleStatus = 'upcoming' | 'active' | 'completed'

export interface Progress {
  completed: number
  total: number
  percent: number
}

export interface InitiativeRef {
  id: string
  name: string
}

export interface Project {
  id: string
  name: string
  description: string
  status: PlanningStatus
  priority: Priority
  startDate: string | null
  targetDate: string | null
  initiative: InitiativeRef | null
  progress: Progress
  color: string
  createdAt: string
  updatedAt: string
  archivedAt: string | null
}

export interface ProjectInput {
  name: string
  description?: string
  status?: PlanningStatus | null
  priority?: Priority | null
  startDate?: string | null
  targetDate?: string | null
  initiativeId?: string | null
  color?: string | null
}

export interface ProjectQuery {
  search?: string | null
  status?: PlanningStatus | null
  initiativeId?: string | null
  includeArchived?: boolean
}

export interface StatusCount {
  status: StatusSummary
  count: number
}

export interface ProjectDetail {
  project: Project
  milestones: Milestone[]
  issueCounts: StatusCount[]
  issueCount: number
}

export interface Milestone {
  id: string
  projectId: string
  name: string
  description: string
  dueDate: string | null
  status: MilestoneStatus
  progress: Progress
  createdAt: string
  updatedAt: string
}

export interface MilestoneInput {
  name: string
  description?: string
  dueDate?: string | null
  status?: MilestoneStatus | null
}

export interface Initiative {
  id: string
  name: string
  description: string
  status: PlanningStatus
  priority: Priority
  startDate: string | null
  targetDate: string | null
  progress: Progress
  projectCount: number
  createdAt: string
  updatedAt: string
  archivedAt: string | null
}

export interface InitiativeInput {
  name: string
  description?: string
  status?: PlanningStatus | null
  priority?: Priority | null
  startDate?: string | null
  targetDate?: string | null
}

export interface InitiativeDetail {
  initiative: Initiative
  projects: Project[]
}

export interface CycleSummary {
  issueCount: number
  completedCount: number
  totalEstimate: number
  completedEstimate: number
  carryOverCount: number
  daysRemaining: number
}

export interface Cycle {
  id: string
  name: string
  startDate: string
  endDate: string
  status: CycleStatus
  summary: CycleSummary
  createdAt: string
  updatedAt: string
}

export interface CycleInput {
  name: string
  startDate: string
  endDate: string
}

export interface CycleDetail {
  cycle: Cycle
  issues: IssueSummary[]
}

export interface Document {
  id: string
  title: string
  body: string
  projectId: string | null
  initiativeId: string | null
  projectName: string | null
  initiativeName: string | null
  archivedAt: string | null
  createdAt: string
  updatedAt: string
}

export interface DocumentInput {
  title: string
  body?: string
  projectId?: string | null
  initiativeId?: string | null
}

export interface Repository {
  id: string
  name: string
  path: string
  provider: 'local' | 'github' | 'gitlab' | 'gitea'
  remoteUrl: string | null
  defaultBranch: string | null
  projectId: string | null
  projectName: string | null
  lastCheckedAt: string | null
  createdAt: string
  updatedAt: string
}

export interface RepositoryInput {
  path: string
  name?: string | null
  projectId?: string | null
}

export interface GitStatus {
  branch: string
  upstream: string | null
  ahead: number
  behind: number
  dirty: boolean
  changes: string[]
}

export interface GitBranch {
  name: string
  current: boolean
}

export interface GitLogEntry {
  hash: string
  shortHash: string
  subject: string
  author: string
  committedAt: string
}

export interface GitDiffStat {
  filesChanged: number
  insertions: number
  deletions: number
  summary: string
}

export interface GitOverview {
  repository: Repository
  status: GitStatus
  branches: GitBranch[]
  log: GitLogEntry[]
  diff: GitDiffStat
}

export interface GitAvailability {
  available: boolean
  version: string | null
  error: string | null
}

export interface BudgetSummary {
  apiFamily: string
  resourceName: string
  limitValue: number | null
  remainingValue: number | null
  usedValue: number | null
  resetAtUtc: string | null
  observedAtUtc: string
  httpStatus: number | null
  source: string
  isEstimate: boolean
  secondaryPauseUntil: string | null
  backgroundAllowance: number
  backgroundReserve: number
  observedRequests: number
}

export interface RequestLog {
  id: string
  providerAccountId: string | null
  operationName: string
  operationCode: string
  apiFamily: string
  resourceName: string
  requestClass: string
  fingerprint: string
  estimatedCost: number
  actualCost: number
  httpStatus: number | null
  errorCategory: string | null
  latencyMs: number | null
  receivedAtUtc: string
  retryAttempt: number
  resultState: string
  rateHeadersJson: string
  payloadBytes: number | null
}

export interface QueueJob {
  id: string
  operationName: string
  operationCode: string
  apiFamily: string
  priority: number
  requestClass: string
  state: string
  attempts: number
  estimatedCost: number
  fingerprint: string
  createdAt: string
  updatedAt: string
  lastError: string | null
}

export interface ApiControl {
  paused: boolean
  reason: string
  updatedAt: string
}

export interface ApiMonitorState {
  connected: boolean
  accountLogin: string | null
  budgets: BudgetSummary[]
  requests: RequestLog[]
  queue: QueueJob[]
  control: ApiControl
  lastObservation: string | null
}

export interface DeviceFlowStart {
  userCode: string
  verificationUri: string
  expiresIn: number
  interval: number
}

export interface GitHubConnection {
  connected: boolean
  login: string
  userId: number
  scopes: string
  expiresAt: string
  refreshExpiresAt: string
  storage: 'keyring' | 'memory'
  storageWarning: string | null
}

export interface DeviceFlowPoll {
  status: 'pending' | 'slow_down' | 'success'
  interval: number
  connection: GitHubConnection | null
}

export interface GitHubRepo {
  id: number
  name: string
  fullName: string
  private: boolean
  defaultBranch: string | null
  htmlUrl: string
  description: string | null
  updatedAt: string | null
  owner: { login: string }
}

export interface GitHubImportResult {
  imported: number
  skipped: number
  projectId: string
  projectName: string
  hasMore: boolean
  pages: number
}

export interface GitHubRef {
  ref: string
  sha: string
}

export interface GitHubPull {
  id: number
  number: number
  title: string
  state: string
  draft: boolean
  user: { login: string } | null
  head: GitHubRef
  base: GitHubRef
  updatedAt: string
  createdAt: string
  htmlUrl: string
  body: string | null
  mergeable: boolean | null
  mergedAt: string | null
  additions: number | null
  deletions: number | null
  changedFiles: number | null
  commits: number | null
  requestedReviewers: Array<{ login: string }>
}

export interface GitHubCommit {
  sha: string
  commit: {
    message: string
    author: { name: string; date: string } | null
  }
  author: { login: string } | null
}

export interface GitHubReview {
  id: number
  user: { login: string } | null
  state: string
  body: string | null
  submittedAt: string | null
  htmlUrl: string | null
}

export interface GitHubCheckRun {
  id: number
  name: string
  status: string
  conclusion: string | null
  startedAt: string | null
  completedAt: string | null
  htmlUrl: string | null
  detailsUrl: string | null
}

export interface GitHubFile {
  filename: string
  status: string
  additions: number
  deletions: number
  changes: number
  patch: string | null
  previousFilename: string | null
}

export interface GitHubReviewComment {
  id: number
  body: string
  path: string
  line: number | null
  originalLine: number | null
  side: string | null
  user: { login: string } | null
  createdAt: string
  htmlUrl: string | null
}

export interface PullRequestDetail {
  pull: GitHubPull
  commits: GitHubCommit[]
  reviews: GitHubReview[]
  checks: GitHubCheckRun[]
  files: GitHubFile[]
  comments: GitHubReviewComment[]
}

export interface GitHubSearchIssue {
  id: number
  number: number
  title: string
  state: string
  htmlUrl: string
  updatedAt: string
  user: { login: string } | null
}

export interface CiOverview {
  defaultBranch: string
  commits: GitHubCommit[]
  checks: GitHubCheckRun[]
}

export interface PullLink {
  id: string
  localIssueId: string
  issueKey: string
  issueTitle: string
  provider: string
  accountId: string
  repositoryId: string
  pullNumber: number
  pullUrl: string
  title: string
  state: string
  createdAt: string
  updatedAt: string
}

export interface PullCapabilities {
  canManage: boolean
  canReview: boolean
}

export interface GitHubMergeResult {
  merged: boolean
  message: string
  sha: string | null
}

export interface InboxEvent {
  id: string
  source: 'local' | 'github' | 'sync'
  kind: string
  title: string
  summary: string
  entityType: string
  entityId: string
  entityKey: string
  entityTitle: string
  createdAt: string
  read: boolean
  archived: boolean
}

export interface InsightBucket {
  label: string
  count: number
}

export interface InsightPoint {
  label: string
  count: number
}

export interface InsightsSummary {
  generatedAt: string
  dataThrough: string
  rangeDays: number
  totalCreated: number
  totalCompleted: number
  openCount: number
  overdueCount: number
  avgCycleDays: number | null
  throughput: InsightPoint[]
  byStatus: InsightBucket[]
  byProject: InsightBucket[]
}
