# Stage 10 report — Stabilization/release (partial)

**Date:** 2026-11-15
**Status:** PARTIAL — performance, migration and accessibility stabilization are
implemented and tested. Native Windows/Linux runtime verification and signed
installers are blocked by the available environment/credentials. Export/import and
full release packaging remain open.
**Schema:** v9

## Delivered

- Route-level code splitting for all screens.
  - Main production JS chunk reduced from ~501 KB to ~231 KB.
  - Vite 500 KB chunk warning is gone.
- Migration upgrade test from v5 to v9:
  - applies historical migrations, inserts a workspace, upgrades to current schema;
  - verifies workspace data and new tables (`pull_request_link`, `inbox_state`,
    `automation_rule`, `dashboard_widget`).
- Accessibility gate assertion:
  - every visible button must have an accessible name (`aria-label`, `title` or text);
  - all 57 visual cases pass.
- Optional AI CI lanes:
  - AI crate tests;
  - default + `VITE_AI_EXTENSION=true` frontend builds;
  - Tauri default and `--features ai-extension` compile checks.
- Release checklist and known limitations documented.

## Blocked / not delivered

- Native Tauri runtime verification on Windows/Linux: local host lacks a runnable
  WebKitGTK desktop session; no Windows runner with interactive desktop.
- Signed installers: no signing certificate/credentials.
- Workspace export/import beyond existing SQLite backup/restore.
- Full crash-report upload/telemetry (offline policy; no backend).
- Performance profiling under large real workspaces and long-running sync.

## Verification

| Check | Command | Result |
|---|---|---|
| Core tests | `cargo test -p forgedesk-core` | PASS — 91 (+3 ignored live) |
| AI crate | `cargo test -p forgedesk-ai` | PASS — 5 |
| Frontend | `npm run typecheck`, `npm test` | PASS — 107 |
| Visual/accessibility | `node scripts/visual-gate0.mjs` | PASS — 57 captures |
| Production chunk | `npm run build` | PASS — main JS ~231 KB |
| Migration v5→v9 | `cargo test ... upgrades_v5` | PASS |
| Native runtime | — | BLOCKED — no desktop WebKitGTK |
| Signed installers | — | BLOCKED — no signing credentials |

## Release blockers

1. Run native Windows and Linux installers/smoke tests on real desktops.
2. Provide signing credentials and produce signed installers.
3. Complete export/import and crash diagnostics policy.
4. Perform large-workspace performance/soak tests.
5. Resolve remaining Stage 5 sync-write limitations if they are in release scope.
