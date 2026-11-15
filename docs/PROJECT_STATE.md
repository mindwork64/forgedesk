# PROJECT_STATE

**Updated:** 2026-11-15  
**Stage:** 2 — Issues/Workflow/Search/Views (core flow delivered; gate PARTIAL)  
**Revision:** working tree / local git repository (see `git log -1`)

## Current objective

Complete the remaining Stage 2 work: UI filter builder over the existing query
engine, Saved Views API/UI, command-palette issue search, list virtualization, then
close UI Gate 1.

## Architecture (locked)

Rust workspace: `crates/forgedesk-core` (domain, SQLite, app services, backups;
no Tauri) + `src-tauri` (typed Tauri 2 shell). Vue 3 + TS + Vite renderer with
typed IPC only. SQLite WAL, serialized writes, append-only migrations through
`user_version`. See `docs/architecture/overview.md` and ADR-0001/0002/0003.

## Implemented

- **Storage:** schema v2 — workflow/status, projects FK target, labels, issue key
  counter, issues, issue labels, relations, comments, activity, saved-view table,
  FTS5 with triggers; v1→v2 migration and existing-workspace seed.
- **Issue domain/service:** create/update/archive/duplicate/batch; `FD-<n>` keys;
  parent cycle checks; project/label/status validation; comments; relations with
  inverse perspective; activity log; filtered/sorted list with FTS search; detail
  payload.
- **Tauri/API:** workflow/status/label/project/list/get/create/update/archive/
  batch/duplicate/comment/relation commands; typed frontend facade.
- **UI:** My Issues list, Board with drag status changes, 440 px detail pane
  (overlay <1280), create dialog, inline property/label/date/estimate editing,
  sub-issues, relations, activity, comments, search, bulk selection/actions,
  archive/duplicate, global Esc close.
- **Stage 1 foundation:** onboarding, settings, backups/restore, themes, navigation,
  command palette, shortcut sheet, en/ru.
- **Tests:** 31 core, 70 frontend, token contract, visual gates (22 captures).
- **Jenkins:** local `ForgeDesk-CI` pipeline from local git mirror (successful build
  for Stage 1; Stage 2 snapshot/build after final commit).

## Verification

| Check | Command | Result |
|---|---|---|
| Core tests | `cargo test -p forgedesk-core` | PASS (31) |
| Core clippy | `cargo clippy -p forgedesk-core --all-targets -- -D warnings` | PASS |
| Shell clippy/typecheck | `cargo clippy/check -p forgedesk` (pkg-config shims) | PASS typecheck; native link unverified |
| Format | `cargo fmt --all --check` | PASS |
| Frontend typecheck | `npm run typecheck` | PASS |
| Frontend tests | `npm test` | PASS (70) |
| Frontend build | `npm run build` | PASS; harness excluded |
| Visual gates | `node scripts/visual-gate0.mjs` (harness build) | PASS — 22 captures (16 Gate 0 + 6 Gate 1) |
| Jenkins build 2 | `ForgeDesk-CI` #2 (Stage 1 snapshot) | SUCCESS — 234 s, artifacts archived |
| Jenkins build 3 | Stage 2 snapshot | Pending commit/mirror push |
| Tauri config | `npx tauri info` | Valid; WebKitGTK absent locally |

## Known issues / limitations

- UI filter builder and Saved Views API/UI are not implemented; backend query/filter
  engine and `saved_view` table exist.
- Command palette is actions/navigation only; no issue entity search.
- No list virtualization; API limit 500 rows.
- Workflow transitions/editor, issue templates, watchers, attachments, milestones/
  cycles fields are not implemented.
- Board drag has no custom keyboard move gesture; detail status select is the
  keyboard alternative.
- No native Tauri binary/run/screenshot in this sandbox; CI Linux/Windows lanes
  and a packaging machine must confirm.
- Jenkins local mirror requires the git daemon; `/home/eloraam` is not readable by
  the Jenkins user, so a real remote should replace the bridge.
- No CI run yet for the Stage 2 tree; trigger after commit.

## Next step

Stage 2 completion slice: filter builder UI (AND/OR one nested level) using
`IssueQuery`, Saved Views create/apply/pin/delete, palette issue search, list
virtualization, then rerun Jenkins + UI Gate 1 and update reports.
