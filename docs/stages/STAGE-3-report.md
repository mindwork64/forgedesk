# Stage 3 report — Projects, planning and documents

**Date:** 2026-11-15  
**Status:** PARTIAL — planning vertical slice is implemented and browser-verified;
templates, cycle board/analytics, roadmap milestones and document import/outline
remain open.  
**Schema:** v3

## Scope delivered

### Backend

- Migration v3: `initiative`, project metadata (initiative FK, color, progress
  method), `milestone`, `cycle`, `document`, `issue_cycle`, `issue_milestone`.
- Project list/query/detail/create/update/archive; project detail includes
  milestone list, per-status issue counts and aggregate progress.
- Milestones create/update, issue linking with same-project validation, progress.
- Initiatives create/update/list/detail with linked projects and weighted progress.
- Cycles create/update/list/detail; derived upcoming/active/completed status;
  summary counts/estimates/carry-over/days remaining; issue assignment/unassignment.
- Documents list/get/create/update/delete; Markdown export string; project/initiative
  links with validation.
- Tests: 36 core, including project/milestone/initiative/cycle progress, query
  filters, document export and migration v3.

### Progress formula

- Project: completed issues / (all non-archived, non-deleted issues − canceled),
  rounded to one decimal; canceled issues are excluded from the denominator and
  completed issues are clamped to the eligible total.
- Initiative: sum of completed/total across linked projects, then the same formula
  (no project double-counting because a project has one initiative).
- Milestone: same formula over linked issues.
- Cycle does not use a percent; it shows counts, estimates and carry-over.

### Frontend

- Projects list: dense table, search, status/initiative filters, archived toggle,
  progress, priority, target date, empty/error/loading states.
- New/edit project dialog (560 px) with status, priority, dates and initiative.
- Project Overview: header, six fixed tabs (`Overview`, `Issues`, `Roadmap`,
  `Documents`, `Repositories`, `Activity`), progress/method, milestones with
  progress and issue linking, issue counts, linked docs, repositories/activity
  honest empty states, Issues tab using the shared issue list.
- Initiatives list + overview with project rows and aggregate progress.
- Cycles page: selector, date range, five summary metrics, cycle issues using the
  shared list, add/remove issue, analytics placeholder.
- Documents page: list pane + Markdown editor with debounced autosave state
  (`Saved`/`Saving…`/`Unsaved changes`), project/initiative links, Markdown copy,
  delete confirmation.
- Roadmap: projects/initiatives timeline with zoom presets, today marker, progress
  fill and honest empty state.
- Navigation/routes updated; project/initiative detail keeps the section active.

## Verification

| Check | Command | Result |
|---|---|---|
| Core tests | `cargo test -p forgedesk-core` | PASS — 36 |
| Core clippy | `cargo clippy -p forgedesk-core --all-targets -- -D warnings` | PASS |
| Format | `cargo fmt --all --check` | PASS |
| Frontend typecheck | `npm run typecheck` | PASS |
| Frontend tests | `npm test` | PASS — 78 |
| Frontend build | `npm run build` | PASS |
| UI visual gates | `node scripts/visual-gate0.mjs` (harness build) | PASS — 38 captures (16 Gate 0 + 13 Gate 1 + 9 Gate 2) |
| Jenkins | `ForgeDesk-CI` build 6 (Stage 3 snapshot) | Pending commit/mirror push |
| Native Tauri run | — | SKIPPED (no WebKitGTK/sudo) |

## Acceptance mapping

| Stage 3 criterion | Status |
|---|---|
| Create initiative with multiple projects | PASS |
| Project with milestones | PASS |
| Cycle with tasks | PASS (assignment/unassignment) |
| Change issue states → progress updates | PASS — backend tests and fixture UI |
| Export/import Markdown | Export PASS (copy Markdown); import NOT implemented |
| Project/initiative hierarchy | PASS |
| Progress aggregation avoids duplicate links | PASS — single initiative FK and join tables |
| Templates | NOT implemented |
| Documents outline/search-in-document | NOT implemented |
| Cycle board/burnup/velocity | NOT implemented |
| Roadmap milestones level / dragging / dependencies | PARTIAL — projects/initiatives + zoom + today marker |
| Screenshot Gate 2 | PARTIAL/PASS for delivered screens |

## Known limitations

- Stage 3 templates and document import/outline/search-in-document are absent.
- Cycle view is list-only; burnup/burndown and velocity are placeholders.
- Roadmap has Projects/Initiatives levels only; milestones level, drag editing and
  dependency overlays are not implemented.
- Project Repositories/Activity and linked PR summaries remain Stage 4+.
- Project archive backend exists but no header `...` menu yet.
- Adding an issue from Project Overview navigates to My Issues without pre-filling
  the project.
- Project/initiative detail uses fixed two-column overview; no compact card system.
- Native runtime verification remains open.

## Next

Stage 4 — Local Git + GitHub connection + API Budget foundation; keep Stage 3
follow-ups (templates/import/cycle analytics/roadmap milestones) in the backlog.
