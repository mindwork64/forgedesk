# Changelog

All notable changes to ForgeDesk are documented here. The project is pre-1.0;
APIs and schema may still evolve with migrations.

## [Unreleased]

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
