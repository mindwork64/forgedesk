# Changelog

All notable changes to ForgeDesk are documented here. The project is pre-1.0;
APIs and schema may still evolve with migrations.

## [Unreleased]

### Stage 7 — Inbox/triage (partial)

- Schema v7 `inbox_state`; normalized local activity Inbox with filters, unread/
  archive actions, grouping and detail pane.
- 81 core tests (+3 ignored live), 107 frontend, 55 visual captures.
- Local Insights aggregates and visual dashboard slice.

### Stage 6 completion

- Schema v6 `pull_request_link`; PR↔local issue link/unlink.
- Permission-aware, confirmation-gated merge/squash/rebase, close and review actions
  with head-SHA pinning and no automatic retry of ambiguous writes.
- Read-only review queue, CI/activity, review comments and unified/split diffs.
- 78 core tests (+3 ignored live), 107 frontend, 53 visual captures.

### Stage 6 — pull requests/checks/reviews (read-only, partial)

- GitHub REST pull list/detail, commits, reviews and check-runs.
- Repository selection for PRs; Open/Closed/All table and 440px detail pane.
- Live read-only test against `cli/cli`; no write/merge/review actions exposed.
- Unified PR file patches in the read-only detail pane.
- 52 core tests (+2 ignored live), 78 frontend, 48 visual captures.

### Stage 5 — GitHub sync safety foundation and auth (partial)

- GitHub App Device Flow auth with keyring/memory token vault, `/user` validation,
  refresh rotation and disconnect; live token+refresh scenario verified.
- Schema v5: remote issue links, sync cursors, idempotent outbox, conflict records.
- Closed outbox state machine including ambiguous-outcome reconciliation; no
  automatic retry of uncertain mutations.
- Remote-link ownership, per-resource cursor and conflict persistence with tests.
- 46 core tests; live import/export remains blocked on GitHub App auth/token vault.

### Stage 4 — Local Git, GitHub foundation and API budgets (partial)

- Schema v4: repositories, provider accounts (no token columns), API budget
  observations, sanitized request log, queue jobs and scheduler control.
- Safe Local Git: status/branches/log/diff/create/switch with dirty guard and
  injection tests; local repository registry/list/detail.
- API budget formula/tests, persistence, manual pause/resume, sanitized diagnostics.
- Repositories, Integrations and API Monitor screens.
- 44 core tests, 78 frontend tests, 43 visual-gate captures.
- Remaining: GitHub App auth/token vault, live HTTP adapters, dispatcher worker,
  repository discovery, provider-backed API Monitor data.

### Stage 3 — Projects, planning and documents (partial)

- Schema v3: initiatives, project metadata, milestones, cycles, documents and
  issue↔cycle/milestone links.
- Project list/overview/create/edit/archive, milestones with progress/issue links,
  initiative hierarchy with weighted progress, cycles with summary/assignment,
  documents with autosave/Markdown copy, roadmap timeline.
- 36 core tests, 78 frontend tests, 38 visual-gate captures.
- Remaining: templates, document import/outline, cycle board/analytics, roadmap
  milestones/drag/dependencies, project repositories/activity.

### Stage 2 — Issues, workflow, search and views

- Schema v2: workflow/status, projects FK target, labels, issue key counter,
  issues, relations, comments, activity, saved-view table, FTS5 triggers.
- Issue create/update/archive/duplicate/batch, `FD-<n>` keys, parent cycle checks,
  labels, relations, comments and activity.
- Server-side whitelisted filter engine, FTS search, sorting and pagination.
- My Issues list, Board with drag status changes, 440 px detail pane, create
  dialog, inline property editing, bulk actions and detail relations/comments.
- Filter builder (AND/OR, one nested group), active chips, sort/group menus.
- Saved Views: persisted filter/sort/groupBy/columns/density, pinning, sidebar
  links, save/update/as-new/delete; command-palette issue search.
- Flat-list virtualization above 100 rows.
- 34 core tests, 76 frontend tests, 29 visual-gate captures.
- Stage 2 browser/core gate PASS; native-window verification and saved-view
  column/density application remain open.

### Stage 1 — Desktop foundation, UI Gate 0

- Tauri 2 + Vue 3 shell with the fixed sidebar/header/status-strip geometry.
- ForgeDesk design tokens for dark and light themes, typography and density.
- Shared primitives: buttons, inputs, selects, fields, switches, segmented
  control, modal, confirm dialog, popover, tooltip, toast, empty/error/loading
  states, keyboard hints.
- Fixed navigation in the order defined by `06_UI_UX_SPEC.md`, collapsible groups,
  icon-rail collapse, persisted sidebar/window state.
- First-run onboarding, settings (general, appearance, workspace, storage,
  shortcuts, advanced) and honest placeholders for later stages.
- Local SQLite database with append-only migrations, typed settings and a single
  workspace record.
- Consistent backups via `VACUUM INTO`, SHA-256 manifests, validation, pruning,
  restore with a safety snapshot and rollback.
- English/Russian localization, command palette, keyboard chords and shortcut
  sheet.
- Core and frontend unit tests; UI Gate 0 screenshot harness and CI lanes.

## [0.1.0] — 2026-11-15

- Initial Stage 0 skeleton and architecture lock.
