# PROJECT_STATE

**Updated:** 2026-11-15  
**Stage:** 3 — Projects/Initiatives/Cycles/Documents (planning vertical slice delivered)  
**Revision:** working tree / local git repository (see `git log -1`)

## Current objective

Run the Stage 3 snapshot through Jenkins, then begin Stage 4 (Local Git + GitHub
connection + API Budget foundation). Stage 3 follow-ups remain tracked below.

## Architecture (locked)

Rust workspace: `crates/forgedesk-core` (domain, SQLite, app services, backups; no
Tauri) + `src-tauri` (typed Tauri 2 shell). Vue 3 + TS + Vite renderer with typed
IPC only. SQLite WAL, serialized writes, append-only migrations through
`user_version`. See `docs/architecture/overview.md` and ADR-0001/0002/0003/0004.

## Implemented

### Stage 3 — planning

- **Schema v3:** initiative, project metadata (initiative FK/color/progress method),
  milestone, cycle, document, issue_cycle, issue_milestone.
- **Projects:** list/query/detail/create/update/archive; progress, milestones,
  issue counts; 560 px create/edit dialog; dense Projects table.
- **Project Overview:** six fixed tabs, progress method, milestones with issue
  linking, issue counts, linked documents, honest repositories/activity empties,
  shared Issues list.
- **Initiatives:** list/detail/create/update, linked projects and weighted progress.
- **Cycles:** list/detail/create/update, derived status, five summary metrics,
  issues, add/remove assignment, analytics placeholder.
- **Documents:** list/create/update/delete, project/initiative links, Markdown
  export (clipboard), debounced autosave with visible state.
- **Roadmap:** projects/initiatives timeline, zoom presets, today marker, progress
  fill.
- **Tests:** 36 core, 78 frontend, 38 visual captures.

### Stage 1–2 (preserved)

- Local-first bootstrap/onboarding/settings/backups/themes/shell.
- Schema v2 issues/workflow/labels/relations/comments/activity/FTS5.
- Issue list/board/detail/create, filters, saved views, grouping, palette search,
  virtualization.

## Verification

| Check | Command | Result |
|---|---|---|
| Core tests | `cargo test -p forgedesk-core` | PASS (36) |
| Core clippy | `cargo clippy -p forgedesk-core --all-targets -- -D warnings` | PASS |
| Shell check/clippy | `cargo check/clippy -p forgedesk` (pkg-config shims) | PASS typecheck; native link unverified |
| Format | `cargo fmt --all --check` | PASS |
| Frontend typecheck | `npm run typecheck` | PASS |
| Frontend tests | `npm test` | PASS (78) |
| Frontend build | `npm run build` | PASS; harness excluded |
| Visual gates | `node scripts/visual-gate0.mjs` | PASS — 38 captures (16 Gate 0 + 13 Gate 1 + 9 Gate 2) |
| Jenkins build 5 | Stage 2 completion `07b5e70` | SUCCESS |
| Jenkins build 6 | Stage 3 snapshot | Pending commit/mirror push |
| Tauri config | `npx tauri info` | Valid; WebKitGTK absent locally |

## Known issues / limitations

- Stage 3 templates, document import/outline/search-in-document are not implemented.
- Cycle screen is list-only; burnup/burndown/velocity are placeholders.
- Roadmap has Projects/Initiatives levels only; milestones level, drag editing and
  dependency overlays are not implemented.
- Project Repositories/Activity and linked PR summaries are Stage 4+.
- Project archive backend exists but no header `...` menu yet.
- “Add issue” from Project Overview opens My Issues without pre-filling the project.
- Issue filters/saved views do not yet expose cycle/milestone fields.
- Native Tauri binary/run/screenshots remain unverified in this sandbox.
- Jenkins mirror requires the local git daemon; rotate the API token after setup.

## Next step

Stage 4 — safe Local Git operations, GitHub connection/credentials, API Budget
Manager, serialized request queue, API Monitor and repository discovery. Keep
Stage 3 follow-ups in the backlog.
