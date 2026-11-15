# ADR-0002 — UI foundation and renderer-only verification harness

**Date:** 2026-11-15  
**Status:** accepted  
**Stage:** 1

## Context

Stage 1 must deliver the exact shell and design system from
`06_UI_UX_SPEC.md`, plus SQLite, settings and backup/restore. The development
sandbox has no WebKitGTK development package and no sudo, so the native Tauri
window cannot be launched here. The frontend still requires screenshot-based
acceptance. The UI also needs a small set of derived values that the spec does not
name explicitly (focus rings, overlays, shadows, soft semantic backgrounds).

## Decisions

1. **CSS custom properties are the only visual source of truth.**
   `src/styles/tokens.css` contains the 17 base colors for both themes and the
   exact geometry/type values. Derived tokens are limited to:
   `--fd-accent-contrast`, `--fd-accent-hover`, `--fd-danger-contrast`,
   `--fd-*-soft` (semantic backgrounds), `--fd-overlay`, scrollbar colors,
   `--fd-focus-ring`, shadows and skeleton colors. A unit test pins all base values
   from the specification.
2. **Hash routing.** `createWebHashHistory()` is used so deep links survive the
   packaged custom protocol without server rewrites. Routes are still internal
   paths (`/settings/storage`) persisted in settings.
3. **Responsive sidebar rule.** The user controls collapse above 1200 px. Below
   1200 px the sidebar is forced to the 64 px rail, matching the minimum-viewport
   requirement. The toggle is disabled in that range with a tooltip.
4. **Shell layout.** Full-height grid with a 24 px status strip spanning the whole
   window; the sidebar sits inside the body region, not beside the status strip.
   This matches the wireframe in §2.5.
5. **Tooltips via a directive.** A single fixed-position overlay supports focus and
   hover, viewport clamping and right-side placement in the collapsed rail. Icon
   buttons always carry `aria-label`; the tooltip supplements, never replaces it.
6. **Dialogs and popovers focus their container first.** Focusing the first
   control (usually the close/copy button) caused its tooltip to cover content.
   The container is focused (`tabindex="-1"`) and Tab enters the dialog trap.
7. **Settings writes are queued.** Optimistic UI is applied immediately, but IPC
   writes are serialized so an older response cannot overwrite a newer value. On
   failure the affected snapshot is restored; if newer writes exist, the store
   re-bootstraps instead of clobbering them.
8. **Placeholder policy.** Unimplemented modules render a page template and an
   explicit "Scheduled for Stage N" state. Where the template needs controls
   (My Issues toolbar, quick create), they are disabled with explanatory tooltips.
   No fake data, no working-looking button with no effect.
9. **Visual harness.** A renderer-only in-memory transport exists solely for
   deterministic screenshots when the native shell is unavailable. It is loaded by
   a compile-time `VITE_VISUAL_HARNESS` branch, verified absent from production
   builds, and documented as test infrastructure. It is not a product feature and
   never claims provider or SQLite activity.
10. **Window state in SQLite settings.** Size, position and maximized state are
    persisted on close. Restore applies position only when it intersects an
    available monitor; otherwise the window is centered.
11. **No UI kit.** Primitives are built in-repo on tokens; Lucide (`@lucide/vue`)
    is the only icon source. No generic theme is imported.
12. **Command palette early.** The palette is introduced in Gate 0 because the
    header requires a search trigger and keyboard-first navigation. Its scope is
    limited to real actions/navigation; entity search stays a Stage 2 feature and
    is not simulated.

## Consequences

- UI acceptance can proceed in environments without WebKitGTK, with a clear
  declaration that native rendering is untested.
- The derived tokens must not proliferate. New colors require an ADR and a
  token-level change, not component-local hex values.
- Hash URLs are less pretty but robust in a desktop bundle.
- Disabled placeholder controls are intentional technical debt for one stage and
  must be replaced, not silently enabled, when Stage 2 lands.
- The harness adds a dev-only bundle chunk; production builds remain free of it.

## Rejected alternatives

- **Playwright against the Tauri webview** — impossible without WebKitGTK here and
  unnecessary for geometry verification.
- **Native screenshot via Xvfb** — no display server, no WebKit libraries; would
  add an unverified dependency.
- **Mock data in production** — forbidden; the harness is build-gated and
  documented.
- **History routing** — would need a fallback server/custom protocol rewrite for
  deep links.
- **Generic popover/tooltip library** — adds styling and accessibility surface
  beyond the small contract we need.
