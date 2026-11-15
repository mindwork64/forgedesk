# PROJECT_STATE

**Updated:** 2026-11-15  
**Stage:** 2 — Issues/Workflow/Search/Views (core + filters/views delivered; browser gate PASS)  
**Revision:** working tree / local git repository (see `git log -1`)

## Current objective

Close the Stage 2 verification loop (Jenkins snapshot, native-run note) and begin
Stage 3: Projects, milestones and cycles, preserving the issue/workflow model.

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
- **View model:** server-side filter tree (AND/OR, one nested group), sort,
  FTS search; Saved Views persisted with name/filter/sort/groupBy/columns/density/
  pin/position; commands and store.
- **List UX:** filter builder + active chips, sort menu, grouping menu, Saved Views
  menu/save dialog/pin/delete, sidebar pinned views, command-palette issue search,
  virtualized flat list above 100 rows.
- **Tauri/API:** workflow/status/label/project/list/get/create/update/archive/
  batch/duplicate/comment/relation commands; typed frontend facade.
- **UI:** My Issues list, Board with drag status changes, 440 px detail pane
  (overlay <1280), create dialog, inline property/label/date/estimate editing,
  sub-issues, relations, activity, comments, search, bulk selection/actions,
  archive/duplicate, global Esc close.
- **Stage 1 foundation:** onboarding, settings, backups/restore, themes, navigation,
  command palette, shortcut sheet, en/ru.
- **Tests:** 34 core, 76 frontend, token contract, visual gates (29 captures).
- **Jenkins:** local `ForgeDesk-CI` pipeline from a git daemon mirror; build 5 is
  green for the completion snapshot with 29 archived captures + measurements.

## Verification

| Check | Command | Result |
|---|---|---|
| Core tests | `cargo test -p forgedesk-core` | PASS (34) |
| Core clippy | `cargo clippy -p forgedesk-core --all-targets -- -D warnings` | PASS |
| Shell clippy/typecheck | `cargo clippy/check -p forgedesk` (pkg-config shims) | PASS typecheck; native link unverified |
| Format | `cargo fmt --all --check` | PASS |
| Frontend typecheck | `npm run typecheck` | PASS |
| Frontend tests | `npm test` | PASS (76) |
| Frontend build | `npm run build` | PASS; harness excluded |
| Visual gates | `node scripts/visual-gate0.mjs` (harness build) | PASS — 29 captures (16 Gate 0 + 13 Gate 1) |
| Jenkins build 2 | `ForgeDesk-CI` #2 (Stage 1 snapshot) | SUCCESS — 234 s, artifacts archived |
| Jenkins build 4 | previous Stage 2 snapshot `3f029a4` | SUCCESS — 67 s, 22 captures archived |
| Jenkins build 5 | completion snapshot `07b5e70` | SUCCESS — 90 s, 34 core tests, 29 captures, 30 artifacts |
| Tauri config | `npx tauri info` | Valid; WebKitGTK absent locally |

## Known issues / limitations

- Saved Views persist `columns`/`density` but the list still uses fixed columns and
  standard density; a column chooser/density toggle is not implemented.
- Filter UI supports eq/neq/contains/range/isEmpty operators; backend `in`/`notIn`
  exists but is not exposed in the editor.
- Flat lists virtualize above 100 rows; grouped lists render all matching rows.
- Workflow transitions/editor, issue templates, watchers, attachments, milestones/
  cycles fields are not implemented.
- Board drag has no custom keyboard move gesture; detail status select is the
  keyboard alternative.
- No native Tauri binary/run/screenshot in this sandbox; CI Linux/Windows lanes
  and a packaging machine must confirm.
- Jenkins local mirror requires the git daemon; `/home/eloraam` is not readable by
  the Jenkins user, so a real remote should replace the bridge.
- Build 3 failed on a stale fixed preview port; `scripts/ci.sh` now allocates a free
  port and verifies the preview process. Builds 4 and 5 are green.

## Next step

Stage 3 — Projects, milestones and cycles, reusing the issue workflow/filter model.
The remaining native-run note closes when a WebKitGTK/WebView2 host is available;
saved-view column/density application and in/notIn filter controls stay in the
follow-up backlog.
