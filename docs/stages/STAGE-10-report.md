# Stage 10 report — Stabilization/release (partial)

**Date:** 2026-11-16
**Status:** PARTIAL — performance, migration and accessibility stabilization are
implemented and tested. CI and a native Windows 10 Debug build are verified. Interactive
Windows/Linux runtime smoke tests, signed installers and full release packaging remain
open.
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
  - all 58 visual cases pass.
- Optional AI CI lanes (the workflow includes the commands and explicitly installs
  the Clippy component; local runs do not substitute for a CI execution result):
  - AI crate tests;
  - default + `VITE_AI_EXTENSION=true` frontend builds;
  - Tauri default and `--features ai-extension` compile checks.
- GitHub Actions CI run
  [43778341394](https://github.com/mindwork64/forgedesk/actions/runs/43778341394)
  passed all seven jobs at `bddc4c58`: Core Ubuntu/Windows, Frontend, Optional AI
  crate, Tauri Linux/Windows compile and UI visual gates.
- Windows 10 Pro x64 Build 19042 native run
  [43778393711](https://github.com/mindwork64/forgedesk/actions/runs/43778393711)
  passed on a self-hosted WIN10 runner with Visual Studio Build Tools 18, MSVC
  14.52.37025, Windows SDK 10.0.26100.0, Rust stable 1.100.0 /
  `x86_64-pc-windows-msvc`, Node.js 22, Git and npm:
  - `npm ci`, typecheck, tests and build;
  - `cargo fmt`, Core Clippy, Core tests and `cargo build -p forgedesk`;
  - `cargo check -p forgedesk --features ai-extension`;
  - Debug `forgedesk.exe` (7,968,310 bytes) uploaded:
    [artifact](https://github.com/mindwork64/forgedesk/actions/runs/43778393711/artifacts/14187497885).
  - Scope caveat: Core tests/Clippy are for `forgedesk-core`, not the full workspace;
    the AI feature was checked, not fully built as a separate AI app; the artifact is
    a Debug executable, not an MSI/release installer; interactive Windows launch and
    user scenarios were not run.
- Temporary `.github/workflows/win10-runner-smoke.yml` and the WIN10 runner were
  used only for this manual check and are removed/deprecated from normal CI.
- Release checklist and known limitations documented.

## Blocked / not delivered

- Interactive native runtime verification on Windows/Linux: the Windows 10 runner
  compiled and tested the Debug executable but did not run the interactive app; Linux
  still lacks a native desktop runtime session.
- Signed installers: no signing certificate/credentials.
- Workspace export/import beyond existing SQLite backup/restore.
- Full crash-report upload/telemetry (offline policy; no backend).
- Performance profiling under large real workspaces and long-running sync.

## Verification

| Check | Command | Result |
|---|---|---|
| Core tests | `cargo test -p forgedesk-core` | PASS — 91 (+3 ignored live) |
| AI crate | `cargo test -p forgedesk-ai` | PASS — 5 |
| Frontend | `npm run typecheck`, `npm test` | PASS — 113 |
| Visual/accessibility | `node scripts/visual-gate0.mjs` | PASS — 58 captures |
| Production chunk | `npm run build` | PASS — main JS ~231 KB |
| Migration v5→v9 | `cargo test ... upgrades_v5` | PASS — data/FK assertions |
| AI Clippy (local) | `cargo clippy -p forgedesk-ai ...` | PASS |
| AI Clippy (CI) | GitHub Actions run `43778341394` | PASS |
| GitHub Actions CI | run `43778341394` at `bddc4c58` | PASS — all 7 jobs |
| Windows 10 native build/tests | run `43778393711` at `bddc4c58` | PASS — Debug exe artifact |
| Native interactive runtime | — | NOT VERIFIED |
| Signed installers | — | BLOCKED — no signing credentials |

## Release blockers

1. Run interactive Windows/Linux smoke tests on real desktops; the Windows Debug
   build itself is verified.
2. Provide signing credentials and produce signed MSI/release artifacts.
3. Complete export/import and crash diagnostics policy.
4. Perform large-workspace performance/soak tests.
5. Resolve remaining Stage 5 sync-write limitations if they are in release scope.
