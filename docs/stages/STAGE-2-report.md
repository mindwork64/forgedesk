## Current-state addendum — 2026-11-18 (full-product audit)

This report records the original Stage 2 delivery snapshot. Current product completion is broader than the browser/core Stage 2 gate. Delivered basics remain valid: Issue CRUD, workflow statuses/categories, labels, relations/sub-issues, comments/activity, List/Board, FTS5, filter builder and Saved Views storage.

Still open for full product DoD: Issue templates, local single-user watchers, managed attachments, Issue milestone/cycle UI, workflow editor/transition rules, Saved View column/density application, in/notIn filter UI, comment edit/soft-delete, Table/Calendar/Timeline views and complete keyboard/accessibility audit. PR #8 completed project archive/restore, key prefixes and Search improvements; PR #11/12 are unrelated to Stage 2.

See docs/specs/PRODUCT_COMPLETION_SPEC.md §4.1 and traceability audit to distinguish DONE from PARTIAL. Native Windows/Linux remains a separate verification task.

---

# Stage 2 report — Issues, workflow, search and views

**Date:** 2026-11-15  
**Status:** COMPLETE for the browser/core Stage 2 scope; native-window verification
and saved-view column/density application remain open.  
**Schema:** v2

## Scope delivered

### Backend

- Migration v2: workflow/status, project FK target, labels, key counter, issues,
  labels links, relations, comments, activity, saved-view table, FTS5 table and
  triggers; default workflow/status seeding for an existing workspace and for new
  workspaces created by the repository.
- Domain: typed issue/status/label/project/comment/activity/filter/query DTOs;
  title/description/estimate/date/label/comment validation; enum parsers; filter
  tree validation.
- Issue service and repositories:
  - create with transactional `FD-<n>` key allocation;
  - update with field clearing, status/project/parent validation and cycle checks;
  - archive/unarchive, duplicate (with related link), batch update;
  - comments, relations (blocks/related/duplicate) with inverse perspective,
    activity records;
  - list with whitelisted SQL filters, FTS5 search, sorting, limit/offset;
  - detail payload with parent, children, relations, comments and activity.
- Tauri commands for workflow, statuses, labels, projects, list/get/create/update/
  archive/batch/duplicate/comment/relation and Saved Views list/create/update/delete.
- Saved Views repository: name/filter/sort/groupBy/columns/density/pin/position,
  input validation, workspace scoping and JSON round-trip.
- Tests: 34 core tests, including migration upgrade, key allocation, hierarchy
  cycle rejection, complex filters, FTS after rename, label/project/clear filters,
  SQL-injection-shaped input, relations, comments, batch updates and Saved Views.

### Frontend

- Real My Issues list (16 deterministic harness rows in screenshots), local search,
  List/Board switcher, bulk selection toolbar with status/priority/archive.
- Board grouped by workflow status; native drag-and-drop changes status; card
  status/detail click; keyboard alternative through the detail status selector.
- Issue Detail Pane (440 px side pane, overlay below 1280 px): editable title,
  description editor, status/priority/type/project/estimate/date/labels editing,
  parent link, sub-issues, relations, activity, comments, duplicate and archive
  confirmation.
- Create Issue dialog (720 px max) with title autofocus, description, status,
  priority, type, project and "More properties" for labels/estimate/dates;
  `Ctrl+Enter` submits.
- Shared issue components: status badge, priority indicator, type icon, label chip,
  row/list/board, detail pane base.
- Store: request-id guards for list/detail, serialized issue mutations, batch
  archive removal, local search debounce; loading/empty/error states.
- Esc closes the detail pane from global focus; Esc inside the pane closes it too.
- Filter builder (AND/OR, one nested group, field/operator/value editor), active
  filter chips, sort menu, group menu, Saved Views menu/save dialog/pin/delete,
  pinned sidebar views.
- Command-palette issue search against the local SQLite/FTS API.
- Flat-list virtualization above 100 rows with retained selection and row actions.
- UI Gate 1 captures: list+detail, light list, board, create dialog, detail activity,
  1100 px overlay, filter builder/applied, grouped list, active view, saved views,
  save dialog, palette issue search.

## Verification

| Check | Command | Result |
|---|---|---|
| Core tests | `cargo test -p forgedesk-core` | PASS — 34 |
| Core clippy | `cargo clippy -p forgedesk-core --all-targets -- -D warnings` | PASS |
| Format | `cargo fmt --all --check` | PASS |
| Frontend typecheck | `npm run typecheck` | PASS |
| Frontend tests | `npm test` | PASS — 76 |
| Frontend build | `npm run build` | PASS |
| UI visual gates | `node scripts/visual-gate0.mjs` (harness build) | PASS — 29 captures (16 Gate 0 + 13 Gate 1) |
| Manual smoke | create → edit title → comment in harness browser | PASS |
| Jenkins | `ForgeDesk-CI` build 4 (previous snapshot `3f029a4`) | SUCCESS — 22 captures, 23 artifacts |
| Jenkins | `ForgeDesk-CI` build 5 (completion snapshot `07b5e70`) | SUCCESS — 90 s, 34 core tests, 29 captures, 30 artifacts |
| Native Tauri run | — | SKIPPED (no WebKitGTK/sudo) |

## Acceptance mapping

| Stage 2 criterion | Status |
|---|---|
| Create/edit/archive issues; batch priority/labels/status | PASS (priority/status/batch; labels via detail/editor) |
| Link issues / relations | PASS |
| Nested sub-issue + cycle prevention | PASS (backend + create-sub-issue UI) |
| Filter by major fields | PASS — status, category, priority, type, project, label, estimate, dates, title, parent, archived, source |
| Complex AND/OR filter | PASS — UI one nested group, backend validated; tests |
| Saved Views restore after restart | PASS — persisted filter/sort/groupBy/pin; columns/density stored but not applied to list |
| Sort and grouping | PASS — updated/created/priority/status/title/due/estimate; status/priority/project/type grouping |
| Restart persistence | Core SQLite persistence tests PASS; native restart NOT run |
| List/Board UI, detail pane, create/edit forms, search | PASS |
| Command palette entity search | PASS — local issue results, keyboard selection, opens detail |
| Loading/empty/error states | PASS for list/detail; stale/pending N/A until providers |
| Keyboard/focus | PASS subset; full audit in Gate 5 |
| Virtualized list above 100 rows | PASS flat list; grouped mode renders all |
| Screenshot-based acceptance for Gate 1 | PASS browser-rendered; native SKIPPED |

## Known limitations

- Saved Views store columns/density but the list uses fixed columns and standard
  density; no column chooser/density control yet.
- Filter editor exposes eq/neq/contains/range/isEmpty; backend `in`/`notIn` has no
  UI.
- Grouped lists do not virtualize; flat lists do.
- No workflow transition rules/editor; status can be selected freely.
- No issue templates, watchers, attachments, milestone/cycle fields yet.
- Board drag has no custom keyboard move gesture (detail status select is the
  keyboard alternative).
- Activity/comment editing and deletion are not exposed.
- Native desktop binary and native screenshots remain unverified in this sandbox.

## Next

Stage 3 — Projects, milestones and cycles, reusing the issue ownership/filter model.
Native-window verification stays open until a WebKitGTK/WebView2 host is available;
saved-view column/density application and in/notIn UI are follow-ups.
