# Testing and release

## Test layers

- **Core:** Rust unit/integration tests for migrations, issue workflow, sync cursor,
  Outbox transitions, dispatcher, budget policy and worker lifecycle.
- **Frontend:** Vitest for stores, filters, routes, tokens and virtualization.
- **Visual:** Playwright UI Gates with screenshot and geometry/content assertions.
- **CI:** GitHub Actions covers frontend, core on Ubuntu/Windows, Tauri compile on
  Linux/Windows, optional AI crate and UI visual gates.

## Current verified baseline

PR #12 merge commit:
`e380c58098878080b7d746f3cad7889effdc2fb6`.

[GitHub Actions run 44166546258](https://github.com/mindwork64/forgedesk/actions/runs/44166546258)
passed all 7 jobs at exact PR head `405f3d4cf4080a49f154d7d29eea53bade4857b4`.

This CI run verifies PR #12's REST dispatcher/pause slice. It does **not** verify
background scheduling, incremental synchronization, automatic Outbox worker,
GraphQL integration or native interactive runtime behavior.

## Visual gates

- Gate 0: shell, themes, onboarding, settings, palette, overlays.
- Gate 1: issues list/board/detail/create/filters/saved views/palette.
- Gate 2: projects, planning, cycles, documents, roadmap.
- Gate 3: repositories, integrations, API Monitor.

Artifacts live in `docs/ui/current/`; reports in `docs/ui/UI-GATE-*-REPORT.md`.

## Release checklist

- No harness or mock data in production build.
- Migrations tested from older schemas; queue/cursor/Outbox survive backup/restore.
- Windows/Linux native run and smoke tests (separate from compile CI).
- Pause/resume, offline/auth-expired/rate-limit and crash recovery scenarios.
- Accessibility/keyboard audit and light-theme parity.
- Signed installers if credentials are available.
- Known limitations documented before tagging.
