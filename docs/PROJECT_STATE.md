# PROJECT_STATE

**Updated:** 2026-11-15  
**Stage:** 2 — Issues/Workflow/Search/Views (next)  
**Revision:** working tree (no commit yet)

## Current objective

Stage 1 foundation is implemented; UI Gate 0 is accepted for the browser-rendered
shell (native-window capture remains pending). Next: implement the local issue
workflow end-to-end with UI Gate 1.

## Architecture (locked)

Rust workspace: `crates/forgedesk-core` (domain, SQLite, app services, backups;
no Tauri) + `src-tauri` (typed Tauri 2 shell). Vue 3 + TS + Vite renderer with
typed IPC only. SQLite WAL, serialized writes, append-only migrations through
`user_version`. See `docs/architecture/overview.md`, ADR-0001/0002.

## Implemented

- **Core storage:** `Database` (WAL/FK/busy timeout, close/reopen/replace),
  migration v1 (`workspace`, `app_setting`, `schema_migration`), typed settings,
  single workspace creation, validation and stable error categories.
- **Backups:** `VACUUM INTO`, SHA-256 manifests, integrity validation, daily
  auto-backup, retention pruning, restore with safety snapshot and rollback.
- **Tauri shell:** bootstrap/workspace/settings/onboarding/backup commands,
  startup-error retention, window geometry persistence, sanitized error payloads,
  minimal capability, icons, CSP.
- **UI Gate 0:** exact shell geometry, dark/light tokens, fixed navigation,
  collapsible rail, header/status strip, shared primitives, onboarding, settings
  (nine sections), storage backup/restore UI, command palette, shortcut sheet,
  toasts, tooltips, dialogs, en/ru locales.
- **Verification:** 20 core tests, 66 frontend tests, token contract tests, visual
  gate script with 16 screenshots, CI workflow (Linux/Windows/visual).

## Verification

| Check | Command | Result |
|---|---|---|
| Core tests | `cargo test -p forgedesk-core` | PASS (20) |
| Core clippy | `cargo clippy -p forgedesk-core --all-targets -- -D warnings` | PASS |
| Shell clippy | `cargo clippy -p forgedesk --all-targets -- -D warnings` (pkg-config shims) | PASS |
| Format | `cargo fmt --all --check` | PASS |
| Shell typecheck | `cargo check -p forgedesk` (pkg-config shims) | PASS typecheck; native link unverified |
| Frontend typecheck | `npm run typecheck` | PASS |
| Frontend tests | `npm test` | PASS (66) |
| Frontend build | `npm run build` | PASS; harness excluded |
| UI Gate 0 | `PLAYWRIGHT_BROWSERS_PATH=.playwright-browsers node scripts/visual-gate0.mjs` | PASS (16 captures) |
| Tauri config | `npx tauri info` | Config valid; WebKitGTK missing locally |

## Known issues / limitations

- Native Tauri binary not linked/launched in this sandbox (no WebKitGTK/sudo);
  CI Linux/Windows lanes must confirm. Native screenshot: SKIPPED.
- UI Gate 0 evidence is browser-rendered from the production bundle via the
  documented, build-gated visual harness.
- Restore accepts managed backup ids only; import of external files is deferred.
- No attachments yet, so backups contain SQLite only.
- My Issues toolbar and quick create are disabled placeholders until Stage 2.
- Command palette is action/navigation only; entity search comes with Stage 2.
- No providers/sync/API budget/Git/AI code exists; those areas are not simulated.
- CI workflow exists but has not run (repository not pushed).
- Settings queue and backup flows are tested at store/core level; real Tauri IPC
  integration test awaits a runnable shell.

## Next step

Stage 2 vertical slice 1: issue schema + workflow statuses/labels + create/edit/
archive + list view and detail pane, then filters/saved views/search and UI Gate 1.
