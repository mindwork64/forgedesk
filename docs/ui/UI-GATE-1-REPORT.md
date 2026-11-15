# UI Gate 1 acceptance report — Issues and views

**Gate:** Issues and Views (`06_UI_UX_SPEC.md` §12)  
**Stage:** 2  
**Date:** 2026-11-15  
**Verdict:** PASS for the browser-rendered Stage 2 scope; native-window capture is
SKIPPED. Saved-view column selection and density are persisted but not yet applied
to the list.

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
- Filter builder: AND/OR root, one nested group, field/operator/value editor,
  active filter chips, clear/apply.
- Sort and grouping menus; grouped list sections with counts.
- Saved Views: save/update/as-new, apply, pin/unpin to sidebar, delete confirmation;
  filter/sort/groupBy persisted; columns/density stored.
- Command palette searches local issues (FTS-backed API), keyboard-selects and opens
  the detail pane.
- Flat list virtualizes above 100 rows; 16-row reference list remains fully visible.
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
| 7 | `gate1-filter-builder-dark-1440.png` | 1440×900 | Filter builder, condition row |
| 8 | `gate1-filter-applied-dark-1440.png` | 1440×900 | Filter chip + reduced list |
| 9 | `gate1-grouped-list-dark-1440.png` | 1440×900 | Status grouping with counts |
| 10 | `gate1-active-view-dark-1440.png` | 1440×900 | Pinned “Active work” view applied |
| 11 | `gate1-saved-views-menu-dark-1440.png` | 1440×900 | Saved Views menu |
| 12 | `gate1-save-view-dialog-dark-1440.png` | 1440×900 | Save view dialog |
| 13 | `gate1-palette-issues-dark-1440.png` | 1440×900 | Palette issue search |

Full run: **29 captures** (16 Gate 0 + 13 Gate 1), `docs/ui/current/measurements.json`,
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
| Filters (AND/OR, one nested group) | PASS | builder + chips; default-status filter returns 4 rows |
| Saved views (save/apply/pin/restore) | PASS with caveat | persist/apply/pin/delete; columns/density stored but list stays fixed |
| Command palette issue search | PASS | 6 local results for “capture”; keyboard opens detail |
| Virtualization for >100 rows | PASS flat list | 250-row component test; grouped mode renders all |
| Full UI Gate 1 screenshot set | PASS | 13 Gate 1 captures; filters/views/palette/grouping included |
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

The browser-rendered Stage 2 interaction model passes visual and behavioral
acceptance: issue CRUD, list/board/detail, filters, saved views, grouping, palette
search and flat-list virtualization are covered. Native-window verification remains
open and is not claimed as passed. Saved-view column selection/density and
in/notIn filter UI remain follow-ups.
