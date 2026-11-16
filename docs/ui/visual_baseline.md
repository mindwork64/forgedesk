# Visual baseline — UI Gates 0–3

**Gates:** UI Gate 0 — Design System + App Shell; UI Gate 1 — Issues and views (Stage 2); UI Gate 2 — Projects/planning/documents (Stage 3); UI Gate 3 — Repositories/Integrations/API Monitor (Stage 4)  
**Date:** 2026-11-15  
**Result:** PASS for the browser-rendered shell; UI Gate 1 PASS; Gate 2 PARTIAL; Gate 3 PARTIAL (local/foundation scope); native-window capture SKIPPED  
**Artifacts:** `docs/ui/current/` (screenshots + `measurements.json`)

## Environment

| Item | Value |
|---|---|
| Browser | Playwright Chromium headless shell 159.0.8154.6 (build 1255) |
| OS | Linux x86_64 (Ubuntu 24.04 userland) |
| Device scale | 1 |
| Fonts | `Inter` is not installed in this sandbox; the documented fallback (`Segoe UI` → system sans) is used. No font files are bundled. |
| Build | Vite production build with `VITE_VISUAL_HARNESS=true` |
| Native shell | Not available in this sandbox: WebKitGTK 4.1 development package and sudo are absent. |

## Harness (test-only, not a product feature)

`scripts/visual-gate0.mjs` loads the built renderer with a deterministic in-memory
IPC transport from `src/dev/visualHarness.ts`. It exists so UI geometry and states
can be reviewed without a native webview. It is gated by `VITE_VISUAL_HARNESS`:

- production `npm run build` was verified to contain no harness string/chunk;
- the harness never runs in a packaged build;
- all product commands still go through the real typed IPC in normal use.

Query parameters used by the harness: `theme=dark|light`,
`sidebar=expanded|collapsed`, `route=/path`, `locale=en|ru`, `onboarding=1`,
`error=1`. These are fixture controls, not app settings.

## Captures

| # | File | Viewport | Theme | Sidebar | State |
|---|---|---|---:|---|---|
| 1 | `gate0-shell-dark-1440-expanded.png` | 1440×900 | Dark | 240 px | My Issues page template |
| 2 | `gate0-shell-light-1440-expanded.png` | 1440×900 | Light | 240 px | My Issues page template |
| 3 | `gate0-shell-dark-1280-collapsed.png` | 1280×800 | Dark | 64 px | icon rail |
| 4 | `gate0-shell-light-1280-collapsed.png` | 1280×800 | Light | 64 px | icon rail |
| 5 | `gate0-shell-dark-1100-collapsed.png` | 1100×700 | Dark | 64 px | minimum viewport, auto-collapsed |
| 6 | `gate0-shell-dark-1440-russian.png` | 1440×900 | Dark | 240 px | `ru` locale |
| 7 | `gate0-settings-storage-dark-1440.png` | 1440×900 | Dark | 240 px | Storage settings, snapshot table |
| 8 | `gate0-restore-dialog-dark-1440.png` | 1440×900 | Dark | 240 px | destructive confirmation dialog |
| 9 | `gate0-onboarding-dark-1440.png` | 1440×900 | Dark | 240 px | first-run step 1 |
| 10 | `gate0-command-palette-dark-1440.png` | 1440×900 | Dark | 240 px | palette opened with Ctrl+K |
| 11 | `gate0-status-popover-dark-1440.png` | 1440×900 | Dark | 240 px | local status popover |
| 12 | `gate0-tooltip-dark-1280.png` | 1280×800 | Dark | 64 px | collapsed-rail tooltip |
| 13 | `gate0-toast-dark-1440.png` | 1440×900 | Dark | 240 px | backup toast after palette action |
| 14 | `gate0-inbox-shortcut-dark-1440.png` | 1440×900 | Dark | 240 px | `G` then `I` navigation |
| 15 | `gate0-startup-error-dark-1440.png` | 1440×900 | Dark | 240 px | startup error surface |
| 16 | `gate0-settings-storage-light-1440.png` | 1440×900 | Light | 240 px | Storage settings, light theme |
| 17 | `gate1-my-issues-list-detail-dark-1440.png` | 1440×900 | Dark | 240 px | 16-row list + FD-101 detail |
| 18 | `gate1-detail-overlay-1100.png` | 1100×700 | Dark | 64 px | detail pane overlay |
| 19 | `gate1-my-issues-list-light-1440.png` | 1440×900 | Light | 240 px | issue list, light |
| 20 | `gate1-my-issues-board-dark-1440.png` | 1440×900 | Dark | 240 px | board, first four columns 4/4/4/3 |
| 21 | `gate1-create-dialog-dark-1440.png` | 1440×900 | Dark | 240 px | create dialog 720 px |
| 22 | `gate1-detail-activity-dark-1440.png` | 1440×900 | Dark | 240 px | relations/activity/comments |
| 23 | `gate1-filter-builder-dark-1440.png` | 1440×900 | Dark | 240 px | filter builder with a condition |
| 24 | `gate1-filter-applied-dark-1440.png` | 1440×900 | Dark | 240 px | filter chip, 4-row result |
| 25 | `gate1-grouped-list-dark-1440.png` | 1440×900 | Dark | 240 px | status grouping + counts |
| 26 | `gate1-active-view-dark-1440.png` | 1440×900 | Dark | 240 px | pinned “Active work” view |
| 27 | `gate1-saved-views-menu-dark-1440.png` | 1440×900 | Dark | 240 px | saved views menu |
| 28 | `gate1-save-view-dialog-dark-1440.png` | 1440×900 | Dark | 240 px | save view dialog |
| 29 | `gate1-palette-issues-dark-1440.png` | 1440×900 | Dark | 240 px | palette issue search |
| 30 | `gate2-projects-dark-1440.png` | 1440×900 | Dark | 240 px | projects table |
| 31 | `gate2-project-dialog-dark-1440.png` | 1440×900 | Dark | 240 px | project dialog 560 px |
| 32 | `gate2-project-overview-dark-1440.png` | 1440×900 | Dark | 240 px | project overview + milestones |
| 33 | `gate2-project-issues-dark-1440.png` | 1440×900 | Dark | 240 px | project issues tab |
| 34 | `gate2-initiatives-dark-1440.png` | 1440×900 | Dark | 240 px | initiatives table |
| 35 | `gate2-initiative-overview-dark-1440.png` | 1440×900 | Dark | 240 px | initiative overview |
| 36 | `gate2-cycles-dark-1440.png` | 1440×900 | Dark | 240 px | active cycle summary |
| 37 | `gate2-documents-dark-1440.png` | 1440×900 | Dark | 240 px | Markdown editor |
| 38 | `gate2-roadmap-dark-1440.png` | 1440×900 | Dark | 240 px | projects timeline |
| 39 | `gate3-repositories-dark-1440.png` | 1440×900 | Dark | 240 px | local repositories |
| 40 | `gate3-repository-detail-dark-1440.png` | 1440×900 | Dark | 240 px | clean repository detail |
| 41 | `gate3-repository-dirty-dark-1440.png` | 1440×900 | Dark | 240 px | dirty worktree guard |
| 42 | `gate3-integrations-dark-1440.png` | 1440×900 | Dark | 240 px | provider integrations |
| 43 | `gate3-api-monitor-dark-1440.png` | 1440×900 | Dark | 240 px | API Monitor budgets/history |
| 44 | `gate3-github-auth-dark-1440.png` | 1440×900 | Dark | 240 px | GitHub Device Flow dialog |
| 45 | `gate3-github-repos-dark-1440.png` | 1440×900 | Dark | 240 px | discovered GitHub repositories/import |
| 46 | `gate3-prs-dark-1440.png` | 1440×900 | Dark | 240 px | pull request table |
| 47 | `gate3-pr-detail-dark-1440.png` | 1440×900 | Dark | 240 px | PR detail pane |
| 48 | `gate3-pr-files-dark-1440.png` | 1440×900 | Dark | 240 px | PR unified file patches |
| 49 | `gate3-pr-files-overlimit-dark-1440.png` | 1440×900 | Dark | 240 px | PR 3000-file cap warning |
| 50 | `gate3-reviews-dark-1440.png` | 1440×900 | Dark | 240 px | requested reviews queue |
| 51 | `gate3-ci-dark-1440.png` | 1440×900 | Dark | 240 px | commits and check-runs |
| 52 | `gate3-pr-files-split-dark-1440.png` | 1440×900 | Dark | 240 px | split PR diff |
| 53 | `gate3-pr-merge-dark-1440.png` | 1440×900 | Dark | 240 px | merge confirmation dialog |
| 54 | `gate4-inbox-dark-1440.png` | 1440×900 | Dark | 240 px | Inbox groups/filters/unread |
| 55 | `gate4-insights-dark-1440.png` | 1440×900 | Dark | 240 px | Insights metrics and charts |
| 56 | `gate4-automations-dark-1440.png` | 1440×900 | Dark | 240 px | automation rules and actions |
| 57 | `gate4-dashboards-dark-1440.png` | 1440×900 | Dark | 240 px | persisted dashboard widgets |
| 58 | `gate0-settings-git-dark-1440.png` | 1440×900 | Dark | 240 px | Settings → Git availability/repositories |
| 59 | `gate2-project-repositories-dark-1440.png` | 1440×900 | Dark | 240 px | Project repositories links/remote-only |
| 60* | `gate4-ai-extension-dark-1440.png` | 1440×900 | Dark | 240 px | optional AI screen (`VITE_AI_EXTENSION=true`) |

## Measured geometry

From `docs/ui/current/measurements.json` (tolerance ±2 px):

| Case | Sidebar | Header | Status strip | Content padding | Body background | Horizontal overflow |
|---|---:|---:|---:|---:|---|---|
| dark 1440 expanded | 240 | 48 | 24 | 24 px | `rgb(14,17,23)` | none |
| light 1440 expanded | 240 | 48 | 24 | 24 px | `rgb(245,246,248)` | none |
| dark 1280 collapsed | 64 | 48 | 24 | 24 px | `rgb(14,17,23)` | none |
| light 1280 collapsed | 64 | 48 | 24 | 24 px | `rgb(245,246,248)` | none |
| dark 1100 collapsed | 64 | 48 | 24 | 24 px | `rgb(14,17,23)` | none |

Gate 1 measurements: 16 baseline list rows; detail pane 440 px at 1440 and 1100
(absolute overlay at 1100); board 5 columns with first four card counts 4/4/4/3;
create dialog 720 px; filter state 1 chip / 4 rows; grouped list 5 sections; saved
views menu 2 rows; palette issue search 6 results; project dialog 560 px; project
overview 6 tabs; 6 project issues; 2 initiatives; 2 linked projects; 5 cycle
metrics; 3 documents; 3 roadmap bars; 2 local repositories; 3 repository branches;
4 provider cards; 3 budget panels; queue rows; device code `TEST-CODE`; no console
errors. Gate 3
API/budget data is harness-only; production starts disconnected/empty. The
component test covers 250 virtual rows (fewer than 30 in DOM).

Additional automated assertions:

- exactly one active navigation item on every measured page;
- base font size 13 px;
- design-token values for sidebar, collapsed rail, header, status and content
  padding match `06_UI_UX_SPEC.md`;
- API Monitor is reachable without clipping at 1440×900 and 1280×800;
- palette width ≤ 640 px; onboarding dialog ≤ 560 px;
- tooltip text, toast presence, `G`→`I` route and restore-dialog title verified;
- no browser console errors in any case.

## Reproduce

```bash
VITE_VISUAL_HARNESS=true npm run build
npm run preview -- --host 127.0.0.1 --port 4173
PLAYWRIGHT_BROWSERS_PATH=.playwright-browsers node scripts/visual-gate0.mjs
```

On a machine with WebKitGTK, capture the native window separately at the same
viewports and themes. Native captures are not claimed here.

## Updating this baseline

- Re-run on every change to shared tokens, AppShell, sidebar, header, status strip
  or settings layout.
- Do not re-run for unrelated feature work inside a screen.
- Replace `docs/ui/current/` artifacts only when the UI Gate is re-accepted; keep
  the report and measurements in sync.
- Never treat the harness as evidence that a native installer works.
