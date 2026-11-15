# UI Gate 2 acceptance report — Projects, planning and documents

**Gate:** Projects/Planning/Documents (`06_UI_UX_SPEC.md` §12)  
**Stage:** 3  
**Date:** 2026-11-15  
**Verdict:** PARTIAL — delivered planning screens pass browser review; templates,
cycle analytics, roadmap milestones and document import/outline are not implemented.

## Delivered

- Projects list with dense table, search, status/initiative filters, archived toggle,
  progress, priority, target date and row-to-overview navigation.
- New/edit project dialog at 560 px with status, priority, dates and initiative.
- Project Overview with fixed tab order and six tabs, progress method, milestones
  (progress + issue linking), issue counts, linked documents, repositories/activity
  empty states and shared Issues list.
- Initiatives list and overview with linked projects and aggregate progress.
- Cycles page with selector/date range, five summary metrics, shared issue list,
  add/remove issue and analytics placeholder.
- Documents split view: list + Markdown editor, autosave state, project/initiative
  links, Markdown copy, delete confirmation.
- Roadmap projects/initiatives timeline with 1/3/6/12-month zoom, today marker and
  progress fill.
- Projects/initiatives sidebar prefix states; en/ru labels; empty/error/loading states.

## Captures (Gate 2 subset)

| # | File | Viewport | State |
|---|---|---|---:|
| 1 | `gate2-projects-dark-1440.png` | 1440×900 | projects table, 3 rows |
| 2 | `gate2-project-dialog-dark-1440.png` | 1440×900 | New project, 560 px |
| 3 | `gate2-project-overview-dark-1440.png` | 1440×900 | overview + milestones |
| 4 | `gate2-project-issues-dark-1440.png` | 1440×900 | project issues tab |
| 5 | `gate2-initiatives-dark-1440.png` | 1440×900 | initiatives table |
| 6 | `gate2-initiative-overview-dark-1440.png` | 1440×900 | linked projects/progress |
| 7 | `gate2-cycles-dark-1440.png` | 1440×900 | active cycle summary + issues |
| 8 | `gate2-documents-dark-1440.png` | 1440×900 | Markdown editor, saved |
| 9 | `gate2-roadmap-dark-1440.png` | 1440×900 | projects timeline |

Full run: **38 captures** (16 Gate 0 + 13 Gate 1 + 9 Gate 2),
`docs/ui/current/measurements.json`, zero assertion failures.

## Acceptance checklist

| Check | Result | Evidence |
|---|---|---|
| Projects list default dense table | PASS | 3 rows, progress/priority/target/updated |
| Project create dialog 560 px | PASS | measured 560 |
| Project overview six tabs fixed order | PASS | 6 tab buttons |
| Project issues tab uses shared list | PASS | 6 EXHI issues |
| Milestones with progress/link issue | PASS | two milestones, progress values |
| Initiative overview with linked projects | PASS | 2 project rows |
| Cycle summary/date range/issues | PASS | 5 metrics, 3 issues |
| Document autosave | PASS | edit → Saved state |
| Markdown export | PASS — copy | clipboard copy from backend Markdown |
| Roadmap time axis/today/zoom | PASS | 3 project bars |
| Cycle board/analytics | NOT implemented | list + placeholder only |
| Roadmap milestones level/drag/dependencies | NOT implemented | projects/initiatives only |
| Templates / document import / outline | NOT implemented | — |
| Native Tauri capture | SKIPPED | no WebKitGTK runtime |

## Composition notes

- Project overview blocks are compact section heading + divider; two columns at
  1440, one column below 1200.
- Tabs use text labels and keep the same content width as the shell.
- Project/Initiative detail routes keep the planning section active in the sidebar.
- Document editor clearly distinguishes Saved/Saving/Unsaved changes; no mock
  network state is shown.
- The harness is build-gated; production builds exclude it (verified by CI).

## Conclusion

The Stage 3 planning vertical slice passes browser visual acceptance for its
delivered scope. The gate remains **PARTIAL** until cycle analytics, roadmap
milestones/drag, templates and document import/outline are delivered. Native Tauri
verification remains open.
