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
| `ui` store | `src/stores/ui.ts` | Palette/shortcut overlays, sidebar groups, viewport width, chord hint |
| `toasts` store | `src/stores/toasts.ts` | Bounded toast stack with timed dismissal |
| IPC facade | `src/lib/ipc.ts` | Typed commands, error normalization, dev-only transport |
| Theme/locale | `src/lib/theme.ts`, `src/i18n/index.ts` | Paint hint, locale/theme resolution |
| Navigation | `src/navigation.ts`, `src/router/index.ts` | Fixed nav order and routes |
| Visual harness | `src/dev/visualHarness.ts` | Visual-gate fixtures only; excluded from production build |
