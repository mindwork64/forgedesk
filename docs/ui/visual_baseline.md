# Visual baseline — UI Gate 0

**Gate:** UI Gate 0 — Design System + App Shell (Stage 1)  
**Date:** 2026-11-15  
**Result:** PASS for the browser-rendered shell; native-window capture SKIPPED  
**Artifacts:** `docs/ui/current/` (screenshots + `gate0-measurements.json`)

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

## Measured geometry

From `docs/ui/current/gate0-measurements.json` (tolerance ±2 px):

| Case | Sidebar | Header | Status strip | Content padding | Body background | Horizontal overflow |
|---|---:|---:|---:|---:|---|---|
| dark 1440 expanded | 240 | 48 | 24 | 24 px | `rgb(14,17,23)` | none |
| light 1440 expanded | 240 | 48 | 24 | 24 px | `rgb(245,246,248)` | none |
| dark 1280 collapsed | 64 | 48 | 24 | 24 px | `rgb(14,17,23)` | none |
| light 1280 collapsed | 64 | 48 | 24 | 24 px | `rgb(245,246,248)` | none |
| dark 1100 collapsed | 64 | 48 | 24 | 24 px | `rgb(14,17,23)` | none |

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
