# Changelog

All notable changes to ForgeDesk are documented here. The project is pre-1.0;
APIs and schema may still evolve with migrations.

## [Unreleased]

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
