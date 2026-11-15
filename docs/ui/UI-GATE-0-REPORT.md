# UI Gate 0 acceptance report

**Gate:** Design System + App Shell (`06_UI_UX_SPEC.md` §12)  
**Stage:** 1  
**Date:** 2026-11-15  
**Verdict:** PASS for the browser-rendered shell / PARTIAL for native-window verification (SKIPPED, not failed)

## What was implemented

- Fixed shell: sidebar 240 px (64 px rail), header 48 px, status strip 24 px,
  content padding 24 px; native title bar remains OS-managed.
- Full dark and light token sets, 13 px base typography, 4 px spacing scale,
  32/28 px controls, 40/36 px list rows, 6/8/10 px radii.
- Sidebar workspace header, all five groups in the contractual order, independent
  group collapse, icon-rail mode with right-side tooltips, bottom-pinned Settings
  and Help/About, accent marker on the active item.
- Header breadcrumbs, search trigger (`Ctrl+K`), disabled Stage-2 quick create,
  local status popover, settings entry.
- Status strip with local database state and a details popover (paths, schema,
  version, latest backup, auto-backup state).
- Shared primitives: buttons, icon buttons, inputs, textarea, select, field,
  switch, segmented control, modal, confirm dialog, popover, tooltip, toast,
  empty/error/loading states, keyboard chips.
- Onboarding (three steps, mandatory), settings (nine fixed sections), storage
  backup/restore UI, About, command palette and shortcut sheet.
- English and Russian catalogues; OS locale detection with English fallback.

## Acceptance checklist

| Check | Result | Evidence |
|---|---|---|
| Shell geometry at 1440×900 and 1280×800 | PASS | Measured 240/64, 48, 24, 24; `measurements.json` |
| No horizontal overflow at 1100×700 | PASS | Measured, `gate0-shell-dark-1100-collapsed.png` |
| Sidebar expanded / collapsed | PASS | Dark + light captures at both widths |
| Header and status strip presence/height | PASS | Measured 48 px and 24 px |
| Dark theme tokens | PASS | Exact token contract test + computed backgrounds |
| Light theme tokens | PASS | Exact token contract test + computed backgrounds |
| Typography scale (13 px base, 11/12/14/16/18/20) | PASS | `src/styles/tokens.spec.ts` |
| Buttons/inputs/selects/switch/segmented | PASS | Settings, onboarding and storage captures |
| Menus/popovers | PASS | Status popover capture; popover component used by header and footer |
| Tooltips | PASS | Collapsed-rail tooltip capture with exact text |
| Dialogs and focus trap | PASS | Onboarding + restore confirmation captures |
| Toasts | PASS | Backup toast capture above status strip |
| Keyboard navigation and focus | PASS | Ctrl+K palette, G→I route, Esc/focus trap, visible focus ring |
| Locale switch and OS detection | PASS | English and Russian captures; system locale fallback exercised |
| Placeholder pages use page template | PASS | My Issues template + planned-module pages |
| Empty state | PASS | My Issues empty state |
| Error state | PASS | Startup error capture; inline field errors implemented |
| Offline/local-only state | PASS | Header and status strip say Local only; no network calls exist |
| Loading state component | PASS | `FdLoadingSkeleton` used in boot; not screenshot-captured |
| Pending/stale sync states | N/A | No provider integration exists in this stage |
| No AI navigation in disabled state | PASS | No AI item in nav model or DOM |
| No fake data presented as live | PASS | Harness is visual-gate only; placeholders are labelled and disabled |
| Screenshot set dark/light × expanded/collapsed | PASS | 16 Gate 0 captures in `docs/ui/current/` (latest full run: 22) |
| Native Tauri window screenshot | SKIPPED | WebKitGTK 4.1 absent; native runtime not launched |
| Tauri shell compile | PARTIAL | `cargo check`/`clippy` pass with local pkg-config shims for typechecking; real link/CI build not run here |
| Frontend typecheck/tests/build | PASS | `npm run typecheck`, `npm test` (66), `npm run build` |
| Core tests/lint/format | PASS | 20 tests, clippy `-D warnings`, `cargo fmt --check` |

## Intentional limitations in this gate

- **My Issues controls are disabled placeholders.** The view switcher (except
  List), Filter/Group/Sort/Saved View and Quick create are visibly disabled with
  tooltips explaining Stage 2. They exist to prove the page template and density;
  no action is simulated.
- **Command palette scope.** It performs real local navigation, theme/sidebar
  toggles, backup creation and opens the shortcut sheet. It does not search
  entities yet; the footer states this explicitly.
- **Density** is the fixed standard density; alternative densities are not part of
  this build.
- **Context menus** are not present until entities exist (Stage 2).
- **Native window.** The renderer is the same production bundle used by Tauri, but
  the native window chrome, WebKit rendering and installer are not verified here.
  CI lanes for Linux/Windows compile the shell; native screenshots must be taken
  on a packaging machine.
- **Harness data** is documented in `docs/ui/visual_baseline.md` and excluded from
  production builds (verified by searching the built bundle).

## Conclusion

UI Gate 0 is accepted for the foundation: geometry, tokens, themes, navigation,
shared components, localization and applicable states match the contract, with
screenshot evidence. Native-window and real installer acceptance remain open and
are not claimed as passed.
