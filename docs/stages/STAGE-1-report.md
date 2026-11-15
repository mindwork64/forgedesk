# Stage 1 report — Desktop/Core/SQLite + UI Gate 0

**Date:** 2026-11-15  
**Status:** DONE (with native-runtime verification explicitly pending)  
**Objective:** Tauri/Vue foundation, SQLite bootstrap, settings, backup/restore,
first-run onboarding and UI Gate 0 acceptance.

## Delivered — backend

- Rust workspace with `forgedesk-core` (default member) and `src-tauri` shell.
- SQLite database at the app data directory; WAL, foreign keys, busy timeout,
  serialized writes, reopen/replace support.
- Append-only migration runner (`user_version` + `schema_migration` audit table);
  schema v1 creates `workspace` and `app_setting`. Newer schemas are refused.
- Typed settings document with defaults, normalization and validation; single
  workspace creation protected by a transaction and a conflict error.
- Consistent backups via `VACUUM INTO`; SHA-256 manifest; size/checksum/integrity
  validation; managed snapshot list; retention pruning.
- Restore: validate → safety snapshot → close → replace → reopen; rollback to the
  previous file if reopen fails. Restore is restricted to managed backup ids.
- Startup maintenance: daily automatic snapshot (configurable), age check, prune;
  failures become a visible warning, never a startup block.
- Typed Tauri commands: bootstrap, create workspace, save settings, complete
  onboarding, save window state, list/create/restore backup.
- Startup error retention: a sanitized error is shown on every bootstrap attempt;
  the database is never silently reset.
- Window geometry persistence/restoration with off-screen protection.
- Minimal Tauri capability (`core:default`), no fs/shell/http plugin.
- App icons, CSP, bundle config, CI workflow for Linux/Windows + visual gate.

## Delivered — UI Gate 0

See `docs/ui/UI-GATE-0-REPORT.md` and `docs/ui/visual_baseline.md`.

- Exact shell geometry and both themes; all base tokens pinned by tests.
- Fixed sidebar groups/order, collapse/expand, icon rail, active marker, pinned
  view slot, bottom utility items.
- Header, breadcrumbs, search trigger, quick-create placeholder, status strip and
  local status details.
- Shared primitives and settings screens; onboarding; command palette; shortcut
  sheet; toasts; tooltips; dialogs.
- English/Russian localization with OS detection and English fallback.
- Honest planned-stage pages; no mock feature is presented as working.
- 16 screenshots covering dark/light, expanded/collapsed, 1440/1280/1100,
  settings, onboarding, palette, popover, tooltip, toast, chord navigation,
  restore dialog and startup error.

## Verification

| Check | Command | Result |
|---|---|---|
| Core tests | `cargo test -p forgedesk-core` | PASS — 20 |
| Core clippy | `cargo clippy -p forgedesk-core --all-targets -- -D warnings` | PASS |
| Shell clippy | `cargo clippy -p forgedesk --all-targets -- -D warnings` (with local pkg-config shims) | PASS |
| Format | `cargo fmt --all --check` | PASS |
| Shell typecheck | `cargo check -p forgedesk` (with local pkg-config shims) | PASS typecheck only; native link not verified |
| Frontend typecheck | `npm run typecheck` | PASS |
| Frontend tests | `npm test` | PASS — 66 |
| Frontend build | `npm run build` | PASS |
| Harness exclusion | grep production `dist/` for harness marker | PASS — absent |
| UI Gate 0 | `node scripts/visual-gate0.mjs` with harness build/preview | PASS — 16 captures, 0 assertion failures |
| Tauri config | `npx tauri info` | PASS parse; WebKitGTK absent |
| Native Tauri run/screenshot | — | SKIPPED — no WebKitGTK/sudo in sandbox |
| CI workflow | `.github/workflows/ci.yml` | Added; not executed locally (no remote) |

## Known limitations

- The native desktop binary was not linked or launched here. `cargo check`/clippy
  used synthetic pkg-config descriptors for WebKitGTK/JSC/libsoup to typecheck the
  shell. CI Linux/Windows lanes must confirm real builds.
- UI screenshots are browser-rendered from the same production bundle; native
  window chrome/WebView rendering has no screenshot yet.
- Backup restore accepts managed backup ids only; external database import is not
  enabled. Attachments do not exist yet, so backups contain only SQLite.
- My Issues toolbar/quick create are disabled placeholders until Stage 2.
- Command palette searches actions/navigation only; entity search is Stage 2.
- No provider integration, sync scheduler, API budget manager, Git or AI code yet.
  Nothing in those areas is simulated.
- No CI run has occurred because the repository is not published to a remote.

## Next

Stage 2 — Issues/Workflow/Search/Views and UI Gate 1: issue schema/migrations,
CRUD, statuses, priorities, labels, relations, sub-issues, activity, list/board,
filters, saved views, FTS5 search, detail pane and functional create/edit flows.
