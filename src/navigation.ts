import type { Component } from 'vue'
import {
  Activity,
  ChartColumn,
  CircleDot,
  CircleHelp,
  Eye,
  FileText,
  FolderGit2,
  FolderKanban,
  Gauge,
  GitPullRequest,
  Inbox,
  LayoutDashboard,
  Layers,
  Map,
  Plug,
  Search,
  Settings,
  Target,
  Timer,
  Workflow,
  Zap,
} from '@lucide/vue'

export type NavGroupId =
  | 'workspace'
  | 'planning'
  | 'development'
  | 'insights'
  | 'system'

export interface NavItem {
  key: string
  path: string
  labelKey: string
  icon: Component
  /** Match nested routes (e.g. Settings sections). */
  prefix?: boolean
}

export interface NavGroup {
  id: NavGroupId
  labelKey: string
  items: NavItem[]
}

/**
 * Fixed navigation contract from 06_UI_UX_SPEC.md §4.1. Order and labels are
 * not configurable in v1; the AI item is intentionally absent until the optional
 * extension is enabled.
 */
export const NAV_GROUPS: NavGroup[] = [
  {
    id: 'workspace',
    labelKey: 'nav.groups.workspace',
    items: [
      { key: 'inbox', path: '/inbox', labelKey: 'nav.inbox', icon: Inbox },
      { key: 'my-issues', path: '/my-issues', labelKey: 'nav.myIssues', icon: CircleDot },
      { key: 'activity', path: '/activity', labelKey: 'nav.activity', icon: Activity },
      { key: 'search', path: '/search', labelKey: 'nav.search', icon: Search },
    ],
  },
  {
    id: 'planning',
    labelKey: 'nav.groups.planning',
    items: [
      { key: 'projects', path: '/projects', labelKey: 'nav.projects', icon: FolderKanban, prefix: true },
      { key: 'initiatives', path: '/initiatives', labelKey: 'nav.initiatives', icon: Target, prefix: true },
      { key: 'cycles', path: '/cycles', labelKey: 'nav.cycles', icon: Timer },
      { key: 'roadmap', path: '/roadmap', labelKey: 'nav.roadmap', icon: Map },
      { key: 'documents', path: '/documents', labelKey: 'nav.documents', icon: FileText },
    ],
  },
  {
    id: 'development',
    labelKey: 'nav.groups.development',
    items: [
      {
        key: 'repositories',
        path: '/repositories',
        labelKey: 'nav.repositories',
        icon: FolderGit2,
      },
      {
        key: 'pull-requests',
        path: '/pull-requests',
        labelKey: 'nav.pullRequests',
        icon: GitPullRequest,
      },
      { key: 'reviews', path: '/reviews', labelKey: 'nav.reviews', icon: Eye },
      { key: 'ci', path: '/ci', labelKey: 'nav.ci', icon: Workflow },
    ],
  },
  {
    id: 'insights',
    labelKey: 'nav.groups.insights',
    items: [
      { key: 'insights', path: '/insights', labelKey: 'nav.insights', icon: ChartColumn },
      {
        key: 'dashboards',
        path: '/dashboards',
        labelKey: 'nav.dashboards',
        icon: LayoutDashboard,
      },
    ],
  },
  {
    id: 'system',
    labelKey: 'nav.groups.system',
    items: [
      { key: 'integrations', path: '/integrations', labelKey: 'nav.integrations', icon: Plug },
      { key: 'automations', path: '/automations', labelKey: 'nav.automations', icon: Zap },
      { key: 'api-monitor', path: '/api-monitor', labelKey: 'nav.apiMonitor', icon: Gauge },
    ],
  },
]

export const UTILITY_NAV: NavItem[] = [
  {
    key: 'settings',
    path: '/settings',
    labelKey: 'nav.settings',
    icon: Settings,
    prefix: true,
  },
  { key: 'about', path: '/about', labelKey: 'nav.about', icon: CircleHelp },
]

export const WORKSPACE_MARK = Layers

/** Saved Views are introduced in Stage 2; the nav renders the section only when populated. */
export interface PinnedView {
  id: string
  name: string
  path: string
}

export const PINNED_VIEWS: PinnedView[] = []
