# Testing and release

## Test layers

- **Core:** Rust unit/integration tests for migrations, issue workflow, planning
  progress, documents, repositories, safe Git and API budget formula.
- **Frontend:** Vitest for stores, filters, routes, tokens and virtualization.
- **Visual:** Playwright UI Gate 0–3 screenshots + geometry/content assertions.
- **CI:** local `scripts/ci.sh` and Jenkins `ForgeDesk-CI`; GitHub Actions runs
  frontend, core, Tauri Linux/Windows compile and UI visual gates.

Current baseline: 44 core tests, 78 frontend tests, 43 visual captures. See
`docs/PROJECT_STATE.md` for current counts.

## Visual gates

- Gate 0: shell, themes, onboarding, settings, palette, overlays.
- Gate 1: issues list/board/detail/create/filters/saved views/palette.
- Gate 2: projects, planning, cycles, documents, roadmap.
- Gate 3: repositories, integrations, API Monitor.

Artifacts live in `docs/ui/current/`; reports in `docs/ui/UI-GATE-*-REPORT.md`.

## Release checklist

- No harness or mock data in production build.
- Migrations tested from older schemas; backup/restore verified.
- Windows/Linux native run and signed installers if credentials are available.
- Accessibility/keyboard audit and light-theme parity.
- Known limitations documented before tagging.
