# Stage 2 report — Issues, workflow, search and views

**Date:** 2026-11-15  
**Status:** PARTIAL — vertical issue flow is implemented; filters/saved views UI,
templates, watchers, attachments and workflow editing remain open.  
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
  archive/batch/duplicate/comment/relation.
- Tests: 31 core tests, including migration upgrade, key allocation, hierarchy
  cycle rejection, complex filters, FTS after rename, label/project/clear filters,
  SQL-injection-shaped input, relations, comments and batch updates.

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
- UI Gate 1 captures: list+detail, light list, board, create dialog, detail activity,
  1100 px overlay.

## Verification

| Check | Command | Result |
|---|---|---|
| Core tests | `cargo test -p forgedesk-core` | PASS — 31 |
| Core clippy | `cargo clippy -p forgedesk-core --all-targets -- -D warnings` | PASS |
| Format | `cargo fmt --all --check` | PASS |
| Frontend typecheck | `npm run typecheck` | PASS |
| Frontend tests | `npm test` | PASS — 70 |
| Frontend build | `npm run build` | PASS |
| UI visual gates | `node scripts/visual-gate0.mjs` (harness build) | PASS — 22 captures (16 Gate 0 + 6 Gate 1) |
| Manual smoke | create → edit title → comment in harness browser | PASS |
| Jenkins | `ForgeDesk-CI` build 2 (Stage 1 snapshot) | SUCCESS; Stage 2 build pending after snapshot |
| Native Tauri run | — | SKIPPED (no WebKitGTK/sudo) |

## Acceptance mapping

| Stage 2 criterion | Status |
|---|---|
| Create/edit/archive issues; batch priority/labels/status | PASS (priority/status/batch; labels via detail/editor) |
| Link issues / relations | PASS |
| Nested sub-issue + cycle prevention | PASS (backend + create-sub-issue UI) |
| Filter by major fields | Backend PASS; UI filter builder NOT implemented |
| Complex AND/OR filter | Backend PASS + test; UI NOT implemented |
| Saved Views restore after restart | Schema exists; API/UI NOT implemented |
| Restart persistence | Core SQLite persistence tests PASS; native restart NOT run |
| List/Board UI, detail pane, create/edit forms, search | PASS |
| Command palette entity search | NOT implemented (navigation/actions only) |
| Loading/empty/error states | PASS for list/detail; stale/pending N/A until providers |
| Keyboard/focus | PASS subset; full audit in Gate 5 |
| Screenshot-based acceptance for Gate 1 | PASS browser-rendered; native SKIPPED |

## Known limitations

- No UI filter builder, saved-view save/apply/pin, or command-palette issue search.
- No workflow transition rules/editor; status can be selected freely.
- No issue templates, watchers, attachments, milestone/cycle fields yet.
- List virtualization is not implemented; the API limit is 500 rows.
- Board drag has no custom keyboard move gesture (detail status select is the
  keyboard alternative).
- Activity/comment editing and deletion are not exposed.
- Native desktop binary and native screenshots remain unverified in this sandbox.

## Next

Stage 2 completion slice: filter builder (AND/OR, one nested level) bound to the
existing query engine, saved-view API/UI with pinning, command-palette issue search,
list virtualization, and UI Gate 1 acceptance report update.
