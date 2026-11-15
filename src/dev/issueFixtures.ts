/**
 * Deterministic issue fixtures for UI Gate screenshots.
 * Loaded only through the build-gated visual harness.
 */
import type {
  ActivityEvent,
  BatchIssueInput,
  CommandName,
  Comment,
  CreateIssueInput,
  IssueDetail,
  IssueQuery,
  IssueRelation,
  IssueSummary,
  FilterCondition,
  FilterNode,
  IssueType,
  Label,
  Priority,
  ProjectRef,
  RelationInputKind,
  SavedView,
  SavedViewInput,
  SortSpec,
  StatusSummary,
  UpdateIssueInput,
  Workflow,
  WorkflowStatus,
} from '@/types/ipc'

const now = Date.now()
const iso = (hoursAgo: number): string =>
  new Date(now - hoursAgo * 3_600_000).toISOString()

const STATUS_DEFS: Array<[string, string, StatusSummary['category'], StatusSummary['color'], string]> = [
  ['backlog', 'Backlog', 'backlog', 'neutral', 'backlog'],
  ['todo', 'Todo', 'unstarted', 'info', 'todo'],
  ['in-progress', 'In Progress', 'started', 'accent', 'in_progress'],
  ['done', 'Done', 'completed', 'success', 'done'],
  ['canceled', 'Canceled', 'canceled', 'neutral', 'canceled'],
]

const statuses: WorkflowStatus[] = STATUS_DEFS.map(
  ([id, name, category, color, systemKey], index) => ({
    id: `st-${id}`,
    name,
    category,
    color,
    position: index,
    systemKey,
    isDefault: index === 0,
  }),
)

const workflow: Workflow = {
  id: 'wf-default',
  name: 'Default',
  isDefault: true,
  statuses,
}

const summaryStatus = (id: string): StatusSummary => {
  const status = statuses.find((candidate) => candidate.id === id) ?? statuses[0]
  return {
    id: status.id,
    name: status.name,
    category: status.category,
    color: status.color,
    position: status.position,
    systemKey: status.systemKey,
  }
}

const labels: Label[] = [
  { id: 'label-capture', name: 'capture', color: 'accent' },
  { id: 'label-bug', name: 'bug', color: 'danger' },
  { id: 'label-ui', name: 'ui', color: 'info' },
  { id: 'label-docs', name: 'docs', color: 'neutral' },
  { id: 'label-perf', name: 'performance', color: 'warning' },
]

const projects: ProjectRef[] = [
  { id: 'project-exhi', name: 'EXHI Capture' },
  { id: 'project-forge', name: 'ForgeDesk' },
  { id: 'project-lab', name: 'Lab' },
]

interface FixtureSpec {
  title: string
  status: string
  priority: Priority
  type: IssueType
  project: string | null
  labels: string[]
  estimate: number | null
  description: string
  due: string | null
}

const specs: FixtureSpec[] = [
  {
    title: 'Improve GIF capture quality',
    status: 'st-in-progress',
    priority: 'high',
    type: 'feature',
    project: 'project-exhi',
    labels: ['capture', 'ui'],
    estimate: 5,
    description:
      'Capture currently drops frames on high-resolution displays. Keep the existing encoder but add a quality preset and verify memory use on 4K screens.',
    due: iso(-48),
  },
  {
    title: 'Fix redirect loop after provider login',
    status: 'st-in-progress',
    priority: 'urgent',
    type: 'bug',
    project: 'project-forge',
    labels: ['bug', 'ui'],
    estimate: 2,
    description:
      'The callback URL is normalized twice, so the state cookie is discarded. Reproduce with a clean webview profile.',
    due: iso(-24),
  },
  {
    title: 'Add keyboard shortcut for quick capture',
    status: 'st-todo',
    priority: 'medium',
    type: 'feature',
    project: 'project-exhi',
    labels: ['capture'],
    estimate: 3,
    description: 'Bind the global capture shortcut and show it in the shortcut sheet.',
    due: null,
  },
  {
    title: 'Document local backup restore flow',
    status: 'st-todo',
    priority: 'low',
    type: 'task',
    project: 'project-forge',
    labels: ['docs'],
    estimate: 1,
    description: 'Explain managed snapshots, safety snapshot and rollback behavior.',
    due: null,
  },
  {
    title: 'Reduce startup work in the renderer',
    status: 'st-backlog',
    priority: 'medium',
    type: 'improvement',
    project: 'project-forge',
    labels: ['performance'],
    estimate: 3,
    description: 'Profile the first paint and lazy-load settings sections.',
    due: null,
  },
  {
    title: 'Verify capture on Wayland',
    status: 'st-todo',
    priority: 'high',
    type: 'research',
    project: 'project-exhi',
    labels: ['capture'],
    estimate: 2,
    description: 'Check portal behavior and fallback permissions on GNOME Wayland.',
    due: null,
  },
  {
    title: 'Clean up old screenshot fixtures',
    status: 'st-backlog',
    priority: 'low',
    type: 'chore',
    project: null,
    labels: [],
    estimate: 1,
    description: 'Remove duplicate fixtures from the test directory.',
    due: null,
  },
  {
    title: 'Add task templates',
    status: 'st-backlog',
    priority: 'medium',
    type: 'feature',
    project: 'project-forge',
    labels: ['ui'],
    estimate: 5,
    description: 'Allow saving an issue as a local template.',
    due: null,
  },
  {
    title: 'Capture preview is too small',
    status: 'st-done',
    priority: 'low',
    type: 'bug',
    project: 'project-exhi',
    labels: ['capture', 'ui'],
    estimate: 1,
    description: 'The preview panel now opens at the height of the capture area.',
    due: null,
  },
  {
    title: 'Remove unused icon dependency',
    status: 'st-done',
    priority: 'none',
    type: 'chore',
    project: 'project-forge',
    labels: [],
    estimate: null,
    description: 'Drop the duplicate icon package and keep Lucide only.',
    due: null,
  },
  {
    title: 'Implement provider rate-limit fixtures',
    status: 'st-backlog',
    priority: 'high',
    type: 'task',
    project: 'project-forge',
    labels: ['performance'],
    estimate: 8,
    description: 'Add fixtures for rest/graphql primary and secondary limits.',
    due: null,
  },
  {
    title: 'Add capture preview sub-issue',
    status: 'st-todo',
    priority: 'medium',
    type: 'task',
    project: 'project-exhi',
    labels: ['capture'],
    estimate: 2,
    description: 'Child issue used to demonstrate hierarchy and progress.',
    due: null,
  },
  {
    title: 'Polish capture overlay layout',
    status: 'st-in-progress',
    priority: 'high',
    type: 'feature',
    project: 'project-exhi',
    labels: ['capture', 'ui'],
    estimate: 3,
    description: 'Align the overlay controls with the new density tokens.',
    due: null,
  },
  {
    title: 'Verify keyboard navigation in the detail pane',
    status: 'st-in-progress',
    priority: 'medium',
    type: 'task',
    project: 'project-forge',
    labels: ['ui'],
    estimate: 2,
    description: 'Tab order, focus restore and Escape behavior across sections.',
    due: null,
  },
  {
    title: 'Update provider documentation links',
    status: 'st-done',
    priority: 'low',
    type: 'task',
    project: 'project-forge',
    labels: ['docs'],
    estimate: 1,
    description: 'Point the integration notes at the current GitHub documentation.',
    due: null,
  },
  {
    title: 'Rejected sync prototype',
    status: 'st-canceled',
    priority: 'none',
    type: 'chore',
    project: null,
    labels: [],
    estimate: null,
    description: 'Kept as a record of the rejected polling implementation.',
    due: null,
  },
]

let nextNumber = 200
const issues: IssueSummary[] = specs.map((spec, index) => {
  const keyNumber = 101 + index
  const status = spec.status
  const project = spec.project
    ? projects.find((candidate) => candidate.id === spec.project) ?? null
    : null
  return {
    id: `issue-${keyNumber}`,
    key: `FD-${keyNumber}`,
    keyNumber,
    title: spec.title,
    status: summaryStatus(status),
    priority: spec.priority,
    issueType: spec.type,
    project,
    labels: labels.filter((label) => spec.labels.includes(label.name)),
    estimate: spec.estimate,
    startDate: null,
    dueDate: spec.due,
    parentId: index === 11 ? 'issue-101' : null,
    source: 'local',
    remoteUrl: null,
    archivedAt: null,
    createdAt: iso(240 - index * 8),
    updatedAt: iso(72 - index * 4),
    completedAt:
      summaryStatus(status).category === 'completed' ? iso(24) : null,
    subIssueCount: index === 0 ? 1 : 0,
  }
})

const descriptions = new Map<string, string>(
  specs.map((spec, index) => [`issue-${101 + index}`, spec.description]),
)

const comments = new Map<string, Comment[]>([
  [
    'issue-101',
    [
      {
        id: 'comment-1',
        issueId: 'issue-101',
        body: 'Tested with a 4K display: frames are dropped when the source is above 30 fps.',
        author: 'local',
        createdAt: iso(30),
        updatedAt: iso(30),
      },
      {
        id: 'comment-2',
        issueId: 'issue-101',
        body: 'Preset prototype is ready; memory stays under 120 MB in the same scenario.',
        author: 'local',
        createdAt: iso(6),
        updatedAt: iso(6),
      },
    ],
  ],
])

const relations: IssueRelation[] = [
  {
    id: 'relation-1',
    kind: 'blocks',
    issue: {
      id: 'issue-102',
      key: 'FD-102',
      title: 'Fix redirect loop after provider login',
      status: summaryStatus('st-in-progress'),
    },
  },
  {
    id: 'relation-2',
    kind: 'related',
    issue: {
      id: 'issue-103',
      key: 'FD-103',
      title: 'Add keyboard shortcut for quick capture',
      status: summaryStatus('st-todo'),
    },
  },
]

const activity = new Map<string, ActivityEvent[]>([
  [
    'issue-101',
    [
      {
        id: 'activity-1',
        issueId: 'issue-101',
        kind: 'issue_created',
        data: { key: 'FD-101', title: 'Improve GIF capture quality' },
        createdAt: iso(240),
      },
      {
        id: 'activity-2',
        issueId: 'issue-101',
        kind: 'status_changed',
        data: { from: 'Todo', to: 'In Progress' },
        createdAt: iso(36),
      },
      {
        id: 'activity-3',
        issueId: 'issue-101',
        kind: 'labels_changed',
        data: { labels: ['capture', 'ui'] },
        createdAt: iso(12),
      },
      {
        id: 'activity-4',
        issueId: 'issue-101',
        kind: 'comment_added',
        data: {},
        createdAt: iso(6),
      },
    ],
  ],
])

function summaryById(id: string): IssueSummary {
  const issue = issues.find((candidate) => candidate.id === id)
  if (!issue) {
    throw { code: 'not_found', message: 'Issue was not found.' }
  }
  return issue
}

function detailFor(id: string): IssueDetail {
  const summary = summaryById(id)
  const children = issues.filter((issue) => issue.parentId === id)
  const parent = summary.parentId ? summaryById(summary.parentId) : null
  return structuredClone({
    summary,
    description: descriptions.get(id) ?? '',
    parent,
    children,
    relations: id === 'issue-101' ? relations : [],
    comments: comments.get(id) ?? [],
    activity: activity.get(id) ?? [],
  })
}

function addActivity(issueId: string, kind: string, data: unknown): void {
  const list = activity.get(issueId) ?? []
  list.push({
    id: `activity-${crypto.randomUUID()}`,
    issueId,
    kind,
    data,
    createdAt: new Date().toISOString(),
  })
  activity.set(issueId, list)
}

function labelForName(name: string): Label {
  const existing = labels.find(
    (label) => label.name.toLowerCase() === name.toLowerCase(),
  )
  if (existing) {
    return existing
  }
  const created: Label = {
    id: `label-${crypto.randomUUID()}`,
    name,
    color: 'neutral',
  }
  labels.push(created)
  return created
}

function updateIssue(id: string, input: UpdateIssueInput): IssueDetail {
  const issue = summaryById(id)
  const beforeStatus = issue.status.id
  if (input.title !== undefined && input.title !== null) {
    issue.title = input.title.trim()
  }
  if (input.description !== undefined && input.description !== null) {
    descriptions.set(id, input.description)
  }
  if (input.statusId) {
    issue.status = summaryStatus(input.statusId)
    issue.completedAt =
      issue.status.category === 'completed' ? new Date().toISOString() : null
  }
  if (input.priority) {
    issue.priority = input.priority
  }
  if (input.issueType) {
    issue.issueType = input.issueType
  }
  if (input.estimate !== undefined && input.estimate !== null) {
    issue.estimate = input.estimate
  }
  if (input.startDate !== undefined && input.startDate !== null) {
    issue.startDate = input.startDate
  }
  if (input.dueDate !== undefined && input.dueDate !== null) {
    issue.dueDate = input.dueDate
  }
  if (input.projectId) {
    issue.project =
      projects.find((project) => project.id === input.projectId) ?? null
  }
  if (input.parentId) {
    issue.parentId = input.parentId
  }
  if (input.labelNames) {
    issue.labels = input.labelNames.map(labelForName)
  }
  for (const field of input.clearFields ?? []) {
    if (field === 'estimate') issue.estimate = null
    if (field === 'startDate') issue.startDate = null
    if (field === 'dueDate') issue.dueDate = null
    if (field === 'projectId') issue.project = null
    if (field === 'parentId') issue.parentId = null
  }
  issue.updatedAt = new Date().toISOString()
  if (beforeStatus !== issue.status.id) {
    addActivity(id, 'status_changed', {
      from: summaryStatus(beforeStatus).name,
      to: issue.status.name,
    })
  } else {
    addActivity(id, 'issue_updated', { fields: ['title'] })
  }
  return detailFor(id)
}

const savedViews: SavedView[] = [
  {
    id: 'view-active',
    name: 'Active work',
    filters: {
      kind: 'condition',
      field: 'priority',
      operator: 'gte',
      value: 'high',
    },
    sort: [{ field: 'priority', direction: 'desc' }],
    groupBy: 'status',
    columns: ['key', 'title', 'status', 'priority', 'project', 'labels', 'updated'],
    density: 'standard',
    isPinned: true,
    position: 0,
    createdAt: iso(240),
    updatedAt: iso(24),
  },
  {
    id: 'view-recent',
    name: 'Recently updated',
    filters: null,
    sort: [{ field: 'updatedAt', direction: 'desc' }],
    groupBy: null,
    columns: ['key', 'title', 'status', 'priority', 'updated'],
    density: 'standard',
    isPinned: false,
    position: 1,
    createdAt: iso(200),
    updatedAt: iso(20),
  },
]

const PRIORITY_RANK: Record<Priority, number> = {
  none: 0,
  low: 1,
  medium: 2,
  high: 3,
  urgent: 4,
}

function filterValue(issue: IssueSummary, field: FilterCondition['field']): unknown {
  switch (field) {
    case 'status':
      return issue.status.id
    case 'statusCategory':
      return issue.status.category
    case 'priority':
      return issue.priority
    case 'type':
      return issue.issueType
    case 'project':
      return issue.project?.id ?? null
    case 'label':
      return issue.labels.map((label) => label.id)
    case 'estimate':
      return issue.estimate
    case 'startDate':
      return issue.startDate
    case 'dueDate':
      return issue.dueDate
    case 'createdAt':
      return issue.createdAt
    case 'updatedAt':
      return issue.updatedAt
    case 'title':
      return issue.title
    case 'parent':
      return issue.parentId
    case 'archived':
      return issue.archivedAt !== null
    case 'source':
      return issue.source
    default:
      return null
  }
}

function matchesFilter(issue: IssueSummary, node: FilterNode): boolean {
  if (node.kind === 'group') {
    const values = node.conditions.map((child) => matchesFilter(issue, child))
    return node.op === 'or' ? values.some(Boolean) : values.every(Boolean)
  }
  const actual = filterValue(issue, node.field)
  const expected = node.value
  const comparable = (value: unknown): number | string => {
    if (node.field === 'priority') {
      return PRIORITY_RANK[String(value) as Priority] ?? 0
    }
    if (node.field === 'estimate') {
      return typeof value === 'number' ? value : -1
    }
    if (node.field === 'archived') {
      return value === true ? 1 : 0
    }
    return typeof value === 'number' || typeof value === 'string' ? value : String(value ?? '')
  }
  switch (node.operator) {
    case 'isEmpty':
      return actual === null || actual === undefined || actual === '' || (Array.isArray(actual) && actual.length === 0)
    case 'isNotEmpty':
      return !(actual === null || actual === undefined || actual === '' || (Array.isArray(actual) && actual.length === 0))
    case 'contains':
      return String(actual ?? '').toLowerCase().includes(String(expected ?? '').toLowerCase())
    case 'eq':
      return Array.isArray(actual)
        ? actual.includes(expected)
        : actual === expected || String(actual) === String(expected)
    case 'neq':
      return Array.isArray(actual)
        ? !actual.includes(expected)
        : actual !== expected && String(actual) !== String(expected)
    case 'gt':
      return comparable(actual) > comparable(expected)
    case 'gte':
      return comparable(actual) >= comparable(expected)
    case 'lt':
      return comparable(actual) < comparable(expected)
    case 'lte':
      return comparable(actual) <= comparable(expected)
    case 'in':
      return Array.isArray(expected) && expected.includes(actual)
    case 'notIn':
      return Array.isArray(expected) && !expected.includes(actual)
    default:
      return true
  }
}

function sortIssues(result: IssueSummary[], sort: SortSpec[] | undefined): IssueSummary[] {
  const first = sort?.[0]
  if (!first) {
    return result.sort((a, b) => b.updatedAt.localeCompare(a.updatedAt))
  }
  const direction = first.direction === 'asc' ? 1 : -1
  const value = (issue: IssueSummary): string | number => {
    switch (first.field) {
      case 'priority':
        return PRIORITY_RANK[issue.priority]
      case 'status':
        return statuses.findIndex((status) => status.id === issue.status.id)
      case 'title':
        return issue.title.toLowerCase()
      case 'estimate':
        return issue.estimate ?? -1
      case 'dueDate':
        return issue.dueDate ?? ''
      case 'createdAt':
        return issue.createdAt
      case 'updatedAt':
      default:
        return issue.updatedAt
    }
  }
  return result.sort((a, b) => {
    const left = value(a)
    const right = value(b)
    if (left === right) return 0
    return left < right ? -direction : direction
  })
}

export function issueFixtureList(): IssueSummary[] {
  return structuredClone(issues)
}

export function handleIssueFixtureCommand<T>(
  command: CommandName,
  args?: Record<string, unknown>,
): { handled: boolean; value?: T } {
  switch (command) {
    case 'list_workflow':
      return { handled: true, value: structuredClone(workflow) as T }
    case 'list_issue_statuses':
      return { handled: true, value: structuredClone(statuses) as T }
    case 'list_labels':
      return { handled: true, value: structuredClone(labels) as T }
    case 'list_projects':
      return { handled: true, value: structuredClone(projects) as T }
    case 'list_saved_views':
      return {
        handled: true,
        value: structuredClone(
          savedViews
            .slice()
            .sort(
              (a, b) =>
                Number(b.isPinned) - Number(a.isPinned) ||
                a.position - b.position,
            ),
        ) as T,
      }
    case 'create_saved_view': {
      const input = (args?.input ?? {}) as SavedViewInput
      const created: SavedView = {
        id: `view-${Date.now()}`,
        name: input.name.trim(),
        filters: input.filters
          ? (JSON.parse(JSON.stringify(input.filters)) as FilterNode)
          : null,
        sort: JSON.parse(JSON.stringify(input.sort ?? [])),
        groupBy: input.groupBy ?? null,
        columns: [...(input.columns ?? [
          'key',
          'title',
          'status',
          'priority',
          'project',
          'labels',
          'updated',
        ])],
        density: input.density ?? 'standard',
        isPinned: input.isPinned ?? false,
        position: savedViews.length,
        createdAt: iso(0),
        updatedAt: iso(0),
      }
      savedViews.push(created)
      return { handled: true, value: structuredClone(created) as T }
    }
    case 'update_saved_view': {
      const id = String(args?.id ?? '')
      const input = (args?.input ?? {}) as SavedViewInput
      const index = savedViews.findIndex((view) => view.id === id)
      if (index < 0) {
        throw { code: 'not_found', message: 'Saved view not found' }
      }
      const existing = savedViews[index]!
      const updated: SavedView = {
        ...existing,
        name: input.name.trim(),
        filters: input.filters
          ? (JSON.parse(JSON.stringify(input.filters)) as FilterNode)
          : null,
        sort: JSON.parse(JSON.stringify(input.sort ?? [])),
        groupBy: input.groupBy ?? null,
        columns: input.columns ? [...input.columns] : existing.columns,
        density: input.density ?? existing.density,
        isPinned: input.isPinned ?? false,
        updatedAt: iso(0),
      }
      savedViews[index] = updated
      return { handled: true, value: structuredClone(updated) as T }
    }
    case 'delete_saved_view': {
      const id = String(args?.id ?? '')
      const index = savedViews.findIndex((view) => view.id === id)
      if (index >= 0) {
        savedViews.splice(index, 1)
      }
      return { handled: true, value: undefined as T }
    }
    case 'list_issues': {
      const query = (args?.query ?? {}) as IssueQuery
      const needle = query.search?.trim().toLowerCase() ?? ''
      let result = issues
        .filter((issue) => (query.includeArchived ? true : !issue.archivedAt))
        .filter((issue) => !needle || issue.title.toLowerCase().includes(needle))
        .filter((issue) => !query.filter || matchesFilter(issue, query.filter))
        .slice()
      result = sortIssues(result, query.sort)
      const offset = query.offset ?? 0
      const limit = query.limit ?? 500
      result = result.slice(offset, offset + limit)
      return { handled: true, value: structuredClone(result) as T }
    }
    case 'get_issue': {
      const id = String(args?.id ?? '')
      return { handled: true, value: detailFor(id) as T }
    }
    case 'create_issue': {
      const input = (args?.input ?? {}) as CreateIssueInput
      const keyNumber = nextNumber++
      const created: IssueSummary = {
        id: `issue-${keyNumber}`,
        key: `FD-${keyNumber}`,
        keyNumber,
        title: input.title.trim(),
        status: summaryStatus(input.statusId ?? statuses[0].id),
        priority: input.priority ?? 'none',
        issueType: input.issueType ?? 'task',
        project: input.projectId
          ? projects.find((project) => project.id === input.projectId) ?? null
          : null,
        labels: (input.labelNames ?? []).map(labelForName),
        estimate: input.estimate ?? null,
        startDate: input.startDate ?? null,
        dueDate: input.dueDate ?? null,
        parentId: input.parentId ?? null,
        source: 'local',
        remoteUrl: null,
        archivedAt: null,
        createdAt: new Date().toISOString(),
        updatedAt: new Date().toISOString(),
        completedAt:
          summaryStatus(input.statusId ?? statuses[0].id).category === 'completed'
            ? new Date().toISOString()
            : null,
        subIssueCount: 0,
      }
      descriptions.set(created.id, input.description ?? '')
      issues.unshift(created)
      if (created.parentId) {
        const parent = summaryById(created.parentId)
        parent.subIssueCount += 1
      }
      addActivity(created.id, 'issue_created', {
        key: created.key,
        title: created.title,
      })
      return { handled: true, value: detailFor(created.id) as T }
    }
    case 'update_issue': {
      const id = String(args?.id ?? '')
      return {
        handled: true,
        value: updateIssue(id, (args?.input ?? {}) as UpdateIssueInput) as T,
      }
    }
    case 'archive_issue': {
      const id = String(args?.id ?? '')
      const archived = Boolean(args?.archived)
      const issue = summaryById(id)
      issue.archivedAt = archived ? new Date().toISOString() : null
      issue.updatedAt = new Date().toISOString()
      addActivity(id, archived ? 'archived' : 'unarchived', { archived })
      return { handled: true, value: detailFor(id) as T }
    }
    case 'batch_update_issues': {
      const ids = (args?.ids ?? []) as string[]
      const input = (args?.input ?? {}) as BatchIssueInput
      const result = ids.map((id) => {
        const patch: UpdateIssueInput = {
          statusId: input.statusId ?? null,
          priority: input.priority ?? null,
          issueType: input.issueType ?? null,
        }
        const detail = updateIssue(id, patch)
        if (input.addLabelNames?.length) {
          const names = new Set(detail.summary.labels.map((label) => label.name))
          input.addLabelNames.forEach((name) => names.add(name))
          detail.summary.labels = [...names].map(labelForName)
          summaryById(id).labels = detail.summary.labels
        }
        if (input.removeLabelNames?.length) {
          const remove = new Set(input.removeLabelNames)
          detail.summary.labels = detail.summary.labels.filter(
            (label) => !remove.has(label.name),
          )
          summaryById(id).labels = detail.summary.labels
        }
        if (input.archive) {
          summaryById(id).archivedAt = new Date().toISOString()
        }
        return detailFor(id)
      })
      return { handled: true, value: structuredClone(result) as T }
    }
    case 'duplicate_issue': {
      const id = String(args?.id ?? '')
      const source = detailFor(id)
      const created = (handleIssueFixtureCommand<IssueDetail>('create_issue', {
        input: {
          title: `${source.summary.title} (copy)`,
          description: source.description,
          statusId: source.summary.status.id,
          priority: source.summary.priority,
          issueType: source.summary.issueType,
          estimate: source.summary.estimate,
          startDate: source.summary.startDate,
          dueDate: source.summary.dueDate,
          projectId: source.summary.project?.id ?? null,
          labelNames: source.summary.labels.map((label) => label.name),
        } satisfies CreateIssueInput,
      }).value) as IssueDetail
      addActivity(id, 'duplicated', { newId: created.summary.id })
      return { handled: true, value: created as T }
    }
    case 'add_issue_comment': {
      const issueId = String(args?.issueId ?? '')
      const body = String(args?.body ?? '')
      const list = comments.get(issueId) ?? []
      list.push({
        id: `comment-${crypto.randomUUID()}`,
        issueId,
        body,
        author: 'local',
        createdAt: new Date().toISOString(),
        updatedAt: new Date().toISOString(),
      })
      comments.set(issueId, list)
      addActivity(issueId, 'comment_added', {})
      return { handled: true, value: detailFor(issueId) as T }
    }
    case 'add_issue_relation': {
      const issueId = String(args?.issueId ?? '')
      const relatedId = String(args?.relatedIssueId ?? '')
      const kind = String(args?.kind ?? 'related') as RelationInputKind
      const target = summaryById(relatedId)
      if (issueId === 'issue-101') {
        relations.push({
          id: `relation-${crypto.randomUUID()}`,
          kind,
          issue: {
            id: target.id,
            key: target.key,
            title: target.title,
            status: target.status,
          },
        })
      }
      addActivity(issueId, 'relation_added', { kind })
      return { handled: true, value: detailFor(issueId) as T }
    }
    case 'remove_issue_relation': {
      const issueId = String(args?.issueId ?? '')
      const relationId = String(args?.relationId ?? '')
      const index = relations.findIndex(
        (relation) => relation.id === relationId,
      )
      if (index >= 0) {
        relations.splice(index, 1)
      }
      addActivity(issueId, 'relation_removed', {})
      return { handled: true, value: detailFor(issueId) as T }
    }
    default:
      return { handled: false }
  }
}
