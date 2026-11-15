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
- Realistic v5→v9 migration data-preservation test:
  - creates workspace/settings before v2 so the workflow backfill path is exercised;
  - seeds default workflow/statuses/key counter, then inserts project, issue, comment,
    document, repository, remote link, outbox and cursor using real v5 schemas;
  - upgrades via the normal runner and verifies field values, relationships and
    `PRAGMA foreign_key_check`; new Stage 6–9 tables are asserted.
- Accessibility validation with axe-core:
  - `button-name` plus `aria-command-name` rules run for every visual case;
  - regression fixtures prove visible text, `aria-label`, `aria-labelledby` and icon
    button names pass, while empty/hidden-only buttons and unnamed `role="button"`
    fail;
  - all 57 visual cases pass.
- Optional AI CI lanes (the workflow includes the commands and explicitly installs
  the Clippy component; local runs do not substitute for a CI execution result):
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
| Migration v5→v9 | `cargo test ... upgrades_v5` | PASS — data/FK assertions |
| AI Clippy (local) | `cargo clippy -p forgedesk-ai ...` | PASS; CI execution tracked separately |
| Native runtime | — | BLOCKED — no desktop WebKitGTK |
| Signed installers | — | BLOCKED — no signing credentials |

## Release blockers

1. Run native Windows and Linux installers/smoke tests on real desktops.
2. Provide signing credentials and produce signed installers.
3. Complete export/import and crash diagnostics policy.
4. Perform large-workspace performance/soak tests.
5. Resolve remaining Stage 5 sync-write limitations if they are in release scope.
