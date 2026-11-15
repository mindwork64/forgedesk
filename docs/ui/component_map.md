# UI component map

Compact map for future iterations. Update when a shared component or its public
props change; do not re-read every component for unrelated work.

## Shell and layout

| Component | Path | Purpose | Key props |
|---|---|---|---|
| `AppShell` | `src/components/shell/AppShell.vue` | Fixed shell grid, skip link, route restoration, global shortcuts, palette/shortcut hosts | none |
| `AppSidebar` | `src/components/shell/AppSidebar.vue` | Workspace header, fixed nav groups, pinned-view slot, utility nav | `collapsed`, `canToggle`; emits `toggle` |
| `SidebarGroup` | `src/components/shell/SidebarGroup.vue` | Collapsible group label and item container | `group`, `collapsed`, `open`; emits `toggle` |
| `SidebarNavItem` | `src/components/shell/SidebarNavItem.vue` | Active/hover nav row, icon-rail tooltip, aria-current | `item`, `collapsed` |
| `AppHeader` | `src/components/shell/AppHeader.vue` | Breadcrumbs, search trigger, quick create, local status popover, settings | none |
| `Breadcrumbs` | `src/components/shell/Breadcrumbs.vue` | Up to two-level internal breadcrumbs, settings sections | none |
| `StatusStrip` | `src/components/shell/StatusStrip.vue` | 24 px local DB/sync status strip and details popover | none |
| `LocalStatusDetails` | `src/components/shell/LocalStatusDetails.vue` | Sanitized DB paths, schema, backup state, local-only note | none |
| `PageHeader` | `src/components/layout/PageHeader.vue` | 20 px title, description, page actions | `title`, `description`; slot `actions` |
| `PageToolbar` | `src/components/layout/PageToolbar.vue` | 40 px view/filter toolbar | slots `left`, `right` |

## Shared primitives

| Component | Path | Notes |
|---|---|---|
| `FdButton` | `src/components/ui/FdButton.vue` | `primary`, `secondary`, `ghost`, `destructive`; 32/28 px; loading |
| `FdIconButton` | `src/components/ui/FdIconButton.vue` | Accessible label required; tooltip defaults to label |
| `FdInput` | `src/components/ui/FdInput.vue` | 32 px, v-model string, invalid state, autofocus |
| `FdTextarea` | `src/components/ui/FdTextarea.vue` | v-model string, rows, invalid state |
| `FdSelect` | `src/components/ui/FdSelect.vue` | Native select with token styling |
| `FdField` | `src/components/ui/FdField.vue` | Label/hint/error; slot props `id`, `describedBy` |
| `FdSwitch` | `src/components/ui/FdSwitch.vue` | role switch, required accessible label, v-model boolean |
| `FdSegmented` | `src/components/ui/FdSegmented.vue` | role radiogroup, arrow/Home/End navigation, v-model string |
| `FdModal` | `src/components/ui/FdModal.vue` | Teleported dialog, focus trap, Esc, body scroll lock, sizes sm/md/lg |
| `FdConfirmDialog` | `src/components/ui/FdConfirmDialog.vue` | Confirmation on `FdModal`, destructive option, busy state |
| `FdPopover` | `src/components/ui/FdPopover.vue` | Anchor popover, click-outside/Esc, trigger focus restore |
| `FdToastHost` | `src/components/ui/FdToastHost.vue` | Bottom-right stack above status strip, auto-dismiss handled by store |
| `FdEmptyState` | `src/components/ui/FdEmptyState.vue` | 28 px icon, compact explanation, action slot |
| `FdErrorState` | `src/components/ui/FdErrorState.vue` | Message, code, details disclosure, retry |
| `FdLoadingSkeleton` | `src/components/ui/FdLoadingSkeleton.vue` | Row-shaped skeleton, reduced-motion aware |
| `FdBadge` | `src/components/ui/FdBadge.vue` | Neutral/accent/success/warning/danger/info chips |
| `FdKbd` | `src/components/ui/FdKbd.vue` | Shortcut key chips |

## Issue components

| Component | Path | Purpose |
|---|---|---|
| `IssueStatusBadge` | `src/components/issues/IssueStatusBadge.vue` | Color dot + localized status name |
| `PriorityIndicator` | `src/components/issues/PriorityIndicator.vue` | Priority icon/label with semantic token color |
| `IssueTypeIcon` | `src/components/issues/IssueTypeIcon.vue` | Type icon with accessible label |
| `LabelChip` | `src/components/issues/LabelChip.vue` | Compact label token chip |
| `IssueRow` | `src/components/issues/IssueRow.vue` | 40 px dense row; checkbox, type/title, status, priority, project, labels, updated |
| `IssueList` | `src/components/issues/IssueList.vue` | Sticky header, row rendering, compact columns when detail is open |
| `IssueBoard` | `src/components/issues/IssueBoard.vue` | Status columns, cards, native drag/drop status change |
| `EntityDetailPane` | `src/components/issues/EntityDetailPane.vue` | Base 440 px side pane / absolute overlay below 1280; Esc close |
| `IssueDetailPane` | `src/components/issues/IssueDetailPane.vue` | Issue title/description/properties/hierarchy/relations/activity/comments |
| `CreateIssueDialog` | `src/components/issues/CreateIssueDialog.vue` | 720 px create dialog, title autofocus, more properties |
| `FilterBuilder` / `FilterGroupEditor` / `FilterConditionRow` | `src/components/issues/` | Typed AND/OR filter editor, one nested group |
| `ActiveFilterChips` | `src/components/issues/ActiveFilterChips.vue` | Removable active-filter chips |
| `SortMenu` / `GroupMenu` | `src/components/issues/` | Sort field/direction and client-side grouping |
| `SavedViewsMenu` / `SaveViewDialog` | `src/components/issues/` | Apply/pin/delete and save/update/as-new |
| `IssueList` virtualization | `src/components/issues/IssueList.vue` | Windowed rows above 100, grouped sections otherwise |

## Planning components

| Component | Path | Purpose |
|---|---|---|
| `ProgressBar` | `src/components/planning/ProgressBar.vue` | Accessible percent/completed/total bar |
| `PlanningStatusBadge` | `src/components/planning/PlanningStatusBadge.vue` | Planned/active/paused/completed/canceled dot + label |
| `ProjectsView` / `ProjectDialog` | `src/features/projects/` | Dense project table, create/edit dialog |
| `ProjectOverviewView` / `MilestoneDialog` | `src/features/projects/` | Six tabs, milestones, issue counts/docs |
| `InitiativesView` / `InitiativeDialog` | `src/features/initiatives/` | Initiative table + create/edit |
| `InitiativeOverviewView` | `src/features/initiatives/` | Objective, aggregate progress, linked projects |
| `CyclesView` / `CycleDialog` | `src/features/cycles/` | Cycle selector, metrics, assignment |
| `DocumentsView` | `src/features/documents/` | List + Markdown editor/autosave |
| `RoadmapView` | `src/features/roadmap/` | Projects/initiatives time axis |

## Git and provider components

| Component | Path | Purpose |
|---|---|---|
| `RepositoriesView` / `AddRepositoryDialog` | `src/features/repositories/` | Local repository table + attach dialog |
| `RepositoryDetailView` | `src/features/repositories/` | Overview/branches/commits, dirty guard |
| `IntegrationsView` / `GitHubDeviceDialog` | `src/features/integrations/` | Provider cards, real Device Flow connect/reauthorize |
| `ApiMonitorView` | `src/features/api-monitor/` | Budgets, queue, sanitized request history |
| `ProgressBar`/`PlanningStatusBadge` | `src/components/planning/` | Stage 3 shared indicators |

## Feature screens

| Component | Path | Purpose |
|---|---|---|
| `MyIssuesView` | `src/features/issues/MyIssuesView.vue` | Page template preview; view switcher/filter controls disabled until Stage 2 |
| `PlaceholderView` | `src/features/placeholder/PlaceholderView.vue` | Honest planned-stage page for unimplemented modules |
| `SettingsView` | `src/features/settings/SettingsView.vue` | Fixed settings sub-navigation and section routing |
| `SettingsGeneral` | `src/features/settings/SettingsGeneral.vue` | Language, timezone, startup, build |
| `SettingsAppearance` | `src/features/settings/SettingsAppearance.vue` | Dark/Light/System theme |
| `SettingsWorkspace` | `src/features/settings/SettingsWorkspace.vue` | Workspace identity, key prefix, workflow stage note |
| `SettingsStorage` | `src/features/settings/SettingsStorage.vue` | Paths, auto-backup, retention, snapshot table, restore confirmation |
| `SettingsShortcuts` | `src/features/settings/SettingsShortcuts.vue` | Implemented/planned shortcut reference |
| `SettingsAdvanced` | `src/features/settings/SettingsAdvanced.vue` | Sanitized diagnostics, reset policy note |
| `SettingsPlaceholder` | `src/features/settings/SettingsPlaceholder.vue` | Git/Integrations/Sync planned sections |
| `SettingsSection` | `src/features/settings/SettingsSection.vue` | Section heading/description/actions |
| `SettingsRow` | `src/features/settings/SettingsRow.vue` | Label/hint + control row; optional stacked layout |
| `AboutView` | `src/features/about/AboutView.vue` | Build/schema/stack/platforms/license, no-network note |
| `CommandPalette` | `src/components/CommandPalette.vue` | Local actions/navigation palette; combobox semantics |
| `ShortcutsDialog` | `src/components/ShortcutsDialog.vue` | Global keyboard reference |
| `OnboardingFlow` | `src/components/OnboardingFlow.vue` | Three-step first run; mandatory |
| `StartupError` | `src/components/StartupError.vue` | Full-screen startup/bridge error surface |

## State and integration

| Module | Path | Purpose |
|---|---|---|
| `app` store | `src/stores/app.ts` | Bootstrap, optimistic serialized settings writes, backups/restore, appearance |
| `issues` store | `src/stores/issues.ts` | List/detail, serialized mutations, request guards, search, board grouping |
| `planning` store | `src/stores/planning.ts` | Projects/initiatives/cycles/documents queries and mutations |
| `git` store | `src/stores/git.ts` | Local repository registry and Git operations |
| `githubAuth` store | `src/stores/githubAuth.ts` | Device Flow polling, connection, refresh, disconnect, selected repo |
| `PullRequestsView` | `src/features/pull-requests/` | Read-only PR table + commits/checks/reviews detail |
| `pullRequests` store | `src/stores/pullRequests.ts` | PR list/detail requests and selection |
| `ReviewsView` | `src/features/reviews/` | Requested-review queue, opens PR detail |
| `CiView` | `src/features/ci/` | Recent commits and lazy check-runs |
| `LinkIssueDialog` / merge/review dialogs | `src/features/pull-requests/` | Local PR link + confirmed review/merge actions |
| `apiMonitor` store | `src/stores/apiMonitor.ts` | Budget state, pause/resume, diagnostics |
| `ui` store | `src/stores/ui.ts` | Palette/shortcut overlays, sidebar groups, viewport width, chord hint |
| `toasts` store | `src/stores/toasts.ts` | Bounded toast stack with timed dismissal |
| IPC facade | `src/lib/ipc.ts` | Typed commands, error normalization, dev-only transport |
| Theme/locale | `src/lib/theme.ts`, `src/i18n/index.ts` | Paint hint, locale/theme resolution |
| Navigation | `src/navigation.ts`, `src/router/index.ts` | Fixed nav order and routes |
| Visual harness | `src/dev/visualHarness.ts`, `src/dev/issueFixtures.ts` | Visual-gate fixtures only; excluded from production build |
| CI | `scripts/ci.sh`, `Jenkinsfile` | Shared local/Jenkins verification; screenshots archived |
| Planning fixtures | `src/dev/planningFixtures.ts` | Build-gated Stage 3 screenshots only |
