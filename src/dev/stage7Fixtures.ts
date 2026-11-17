import type {
  AutomationEvent,
  AutomationRule,
  AutomationRun,
  Dashboard,
  CommandName,
  InboxEvent,
  InsightsSummary,
  ProviderCapabilities,
} from '@/types/ipc'

function iso(hoursAgo: number): string {
  return new Date(Date.now() - hoursAgo * 3_600_000).toISOString()
}

const events: InboxEvent[] = [
  {
    id: 'inbox-1',
    source: 'github',
    kind: 'status_changed',
    title: 'Status changed',
    summary: 'Todo → In Progress',
    entityType: 'issue',
    entityId: 'issue-101',
    entityKey: 'FD-141',
    entityTitle: 'GitHub pull request workspace',
    createdAt: iso(1),
    read: false,
    archived: false,
  },
  {
    id: 'inbox-2',
    source: 'local',
    kind: 'comment_added',
    title: 'Comment added',
    summary: 'Design review notes',
    entityType: 'issue',
    entityId: 'issue-102',
    entityKey: 'FD-138',
    entityTitle: 'Repository discovery dialog',
    createdAt: iso(5),
    read: false,
    archived: false,
  },
  {
    id: 'inbox-3',
    source: 'github',
    kind: 'labels_changed',
    title: 'Labels changed',
    summary: 'needs-triage',
    entityType: 'issue',
    entityId: 'issue-103',
    entityKey: 'FD-127',
    entityTitle: 'Sync cursor watermark',
    createdAt: iso(27),
    read: true,
    archived: false,
  },
]

const insights: InsightsSummary = {
  generatedAt: iso(0),
  dataThrough: iso(0),
  rangeDays: 30,
  totalCreated: 38,
  totalCompleted: 24,
  openCount: 14,
  overdueCount: 3,
  avgCycleDays: 4.6,
  throughput: [2, 4, 3, 6, 5, 7, 8, 6, 9, 7, 10, 8].map((count, index) => ({
    label: `W${index + 1}`,
    count,
  })),
  byStatus: [
    { label: 'Todo', count: 5 },
    { label: 'In Progress', count: 4 },
    { label: 'In Review', count: 3 },
    { label: 'Done', count: 24 },
  ],
  byProject: [
    { label: 'ForgeDesk core', count: 18 },
    { label: 'Integrations', count: 11 },
    { label: 'Design system', count: 9 },
  ],
}

const automationRules: AutomationRule[] = [
  {
    id: 'rule-1',
    name: 'Label new issues',
    enabled: true,
    triggerEvent: 'issue_created',
    actionType: 'add_label',
    actionValue: 'automated',
    lastRunAt: iso(2),
    lastResult: 'Applied to 1 event(s)',
    runCount: 4,
    createdAt: iso(100),
    updatedAt: iso(2),
  },
  {
    id: 'rule-2',
    name: 'Mark status changes',
    enabled: false,
    triggerEvent: 'status_changed',
    actionType: 'add_label',
    actionValue: 'status-changed',
    lastRunAt: null,
    lastResult: null,
    runCount: 0,
    createdAt: iso(80),
    updatedAt: iso(80),
  },
]

const automationPreview: AutomationEvent[] = [
  {
    eventId: 'event-1',
    entityId: 'issue-101',
    entityKey: 'FD-141',
    entityTitle: 'GitHub pull request workspace',
    createdAt: iso(1),
  },
]

const automationRuns: AutomationRun[] = [
  {
    id: 'run-1',
    ruleId: 'rule-1',
    eventId: 'event-1',
    status: 'success',
    message: 'Added label automated',
    createdAt: iso(2),
  },
]

const dashboard: Dashboard = {
  id: 'dashboard-1',
  name: 'Overview',
  widgets: [
    { id: 'widget-1', dashboardId: 'dashboard-1', kind: 'metric_open', title: 'Open issues', width: 3, position: 0 },
    { id: 'widget-2', dashboardId: 'dashboard-1', kind: 'issues_recent', title: 'Recent issues', width: 6, position: 1 },
    { id: 'widget-3', dashboardId: 'dashboard-1', kind: 'api_budget', title: 'API budget', width: 3, position: 2 },
  ],
  updatedAt: iso(1),
}

const providerCapabilities: ProviderCapabilities = {
  repositories: true,
  issues: true,
  mergeRequests: true,
  comments: false,
  reviews: false,
  checks: false,
  write: false,
  readOnly: true,
}

export function handleStage7FixtureCommand<T>(
  command: CommandName,
  args?: Record<string, unknown>,
): { handled: boolean; value?: T } {
  switch (command) {
    case 'inbox_list':
    case 'activity_list':
      return { handled: true, value: structuredClone(events) as T }
    case 'inbox_mark_read':
      return { handled: true, value: undefined as T }
    case 'inbox_archive':
      return { handled: true, value: undefined as T }
    case 'inbox_mark_all_read':
      return { handled: true, value: events.filter((event) => !event.read).length as T }
    case 'get_insights':
      return { handled: true, value: structuredClone(insights) as T }
    case 'automation_list_rules':
      return { handled: true, value: structuredClone(automationRules) as T }
    case 'automation_create':
      return {
        handled: true,
        value: structuredClone(automationRules[0]) as T,
      }
    case 'automation_toggle':
      return {
        handled: true,
        value: structuredClone(automationRules[0]) as T,
      }
    case 'automation_delete':
      return { handled: true, value: undefined as T }
    case 'automation_preview':
      return { handled: true, value: structuredClone(automationPreview) as T }
    case 'automation_run':
      return { handled: true, value: 1 as T }
    case 'automation_runs':
      return { handled: true, value: structuredClone(automationRuns) as T }
    case 'dashboard_get':
      return { handled: true, value: structuredClone(dashboard) as T }
    case 'provider_capabilities':
      return { handled: true, value: structuredClone(providerCapabilities) as T }
    case 'ai_dispatch': {
      const action = String(args?.action ?? 'status')
      const payload = (args?.payload ?? {}) as Record<string, unknown>
      const status = {
        featureEnabled: true,
        runtimeEnabled: Boolean(payload.enabled),
        provider: payload.provider ?? 'openai',
        baseUrl: payload.baseUrl ?? 'https://api.openai.com/v1',
        model: payload.model ?? 'gpt-4o-mini',
        hasKey: false,
      }
      if (action === 'generate') {
        return {
          handled: true,
          value: {
            text: 'Harness suggestion: add acceptance criteria and a test note.',
            contextPreview: 'Harness context preview',
          } as T,
        }
      }
      return { handled: true, value: status as T }
    }
    case 'dashboard_add_widget':
    case 'dashboard_remove_widget':
    case 'dashboard_update_width':
    case 'dashboard_reorder':
      return { handled: true, value: structuredClone(dashboard) as T }
    default:
      return { handled: false }
  }
}
