# UI Gate 1 acceptance report — Issues and views

**Gate:** Issues and Views (`06_UI_UX_SPEC.md` §12)  
**Stage:** 2  
**Date:** 2026-11-15  
**Verdict:** PARTIAL — core list/board/detail/create/search flow passes; filters,
saved views and command-palette entity search are not implemented yet.

## Delivered

- My Issues List with real local rows, sticky column header, 40 px density,
  checkbox selection and active-row highlight.
- My Issues Board grouped by workflow status, status/count header, compact cards,
  native drag/drop status change and detail click.
- Issue Detail Pane at 440 px (side pane at ≥1280 px, overlay below 1280 px):
  editable title, Markdown-style plain description editor, status/priority/type/
  project/estimate/date/labels editing, parent, sub-issues, relations, activity,
  comments, duplicate and archive confirmation.
- Create Issue dialog (720 px max): title autofocus, description, status, priority,
  type, project and expanded labels/estimate/dates; `Ctrl+Enter` submits.
- Search in view backed by local FTS5; debounced input; empty/loading/error states.
- Bulk toolbar: selected count, status, priority, archive confirmation, clear.
- Keyboard: `Ctrl+K` palette, `G` chords, `?` help, `Esc` closes the detail pane
  (global and inside the pane), visible focus states.
- English/Russian labels for all new UI, default workflow status names localized
  through `system_key`.

## Captures (Gate 1 subset)

| # | File | Viewport | State |
|---|---|---|---:|
| 1 | `gate1-my-issues-list-detail-dark-1440.png` | 1440×900 | List + FD-101 detail open, 16 rows |
| 2 | `gate1-detail-overlay-1100.png` | 1100×700 | Detail pane overlay at minimum viewport |
| 3 | `gate1-my-issues-list-light-1440.png` | 1440×900 | Light list |
| 4 | `gate1-my-issues-board-dark-1440.png` | 1440×900 | Board, 5 status columns (first four 4/4/4/3 cards) |
| 5 | `gate1-create-dialog-dark-1440.png` | 1440×900 | Create dialog, 720 px |
| 6 | `gate1-detail-activity-dark-1440.png` | 1440×900 | Relations, activity and comments after scroll |

Full run: **22 captures** (16 Gate 0 + 6 Gate 1), `docs/ui/current/measurements.json`,
zero assertion failures.

## Acceptance checklist

| Check | Result | Evidence |
|---|---|---|
| My Issues List at 1440×900 with ≥12 rows | PASS | 16 rows; `issueRows: 16` |
| Detail pane opens at 440 px, list state preserved | PASS | `detailPane.width: 440` |
| Detail pane overlay below 1200 px | PASS | computed `position: absolute` at 1100 |
| Board with ≥4 status columns, ≥3 cards in first four | PASS | `{columns:5, cards:[4,4,4,3]}` |
| Create dialog ≤720 px, title autofocused | PASS | `createDialogWidth: 720` |
| Create → edit → comment flow | PASS | browser smoke; store tests |
| Archive / duplicate / batch actions | PASS | backend + store tests; bulk UI |
| Relations/sub-issues/comments/activity | PASS | detail capture + core tests |
| Local search | PASS | FTS query + store test |
| Loading/empty/error states | PASS | list store state machine; empty action |
| Offline/local-only state | PASS | header/status strip, no network calls |
| Keyboard navigation, focus, Esc | PASS | palette/chord/help/detail close |
| Filters (AND/OR, one nested group) | FAIL — backend only | UI builder not implemented |
| Saved views (save/apply/pin/restore) | FAIL — schema only | API/UI not implemented |
| Command palette issue search | FAIL | palette is actions/navigation only |
| Virtualization for >100 rows | FAIL | not implemented; limit 500 |
| Full UI Gate 1 screenshot set | PARTIAL | 6 Gate 1 captures; filter/saved-view states absent |
| Native Tauri window capture | SKIPPED | WebKitGTK absent; no native runtime |

## Composition notes

- List columns follow the spec order: checkbox, Issue, Title, Status, Priority,
  Project, Labels, Updated; Project/Labels collapse when the detail pane reduces
  the list width.
- The detail pane uses the same header/close/scroll behavior as the base entity
  pane and does not reset list scroll or filters.
- The board shows four full columns at the reference viewport; the fifth
  (Canceled) is reachable by horizontal scroll inside the board only.
- No mock data is shipped: the harness used for screenshots is build-gated and
  excluded from production (verified by `scripts/ci.sh`).

## Conclusion

The core Stage 2 interaction model passes visual and behavioral acceptance. The
gate is intentionally marked **PARTIAL** until filters, saved views, palette entity
search, virtualization and the native-window capture are delivered. Native runtime
verification remains open and is not claimed as passed.
