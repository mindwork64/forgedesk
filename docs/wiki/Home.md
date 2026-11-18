# ForgeDesk Wiki

ForgeDesk is a local-first desktop workspace for issues, projects, cycles,
documents and provider-backed development workflows. It runs on Windows and Linux
with Rust (Tauri 2), Vue 3 + TypeScript and SQLite.

## Start here

- [Architecture](Architecture)
- [Development](Development)
- [Project stages](Project-Stages)
- [GitHub synchronization](GitHub-Synchronization)
- [Testing and release](Testing-and-Release)
- [Security and privacy](Security-and-Privacy)

## Current state — 2026-11-18

- Main revision: `e380c58098878080b7d746f3cad7889effdc2fb6`, schema v11.
- PR #12 merged: shared authenticated GitHub REST dispatcher, mutation pacing,
  REST telemetry/rate observations and an enforced API Monitor Pause/Resume gate.
- PR #12 CI: [run 44166546258](https://github.com/mindwork64/forgedesk/actions/runs/44166546258),
  7/7 jobs passed at the exact PR head.
- Background scheduler, automatic Outbox worker, GraphQL dispatch integration,
  adaptive background quota enforcement and active cursor/watermark sync remain P0 work.
- Native Windows/Linux interactive smoke tests are still separate from CI.

The source-of-truth implementation handoff is
[docs/implementation/P0_REMAINDER_DEEPSEEK_TZ.md](../implementation/P0_REMAINDER_DEEPSEEK_TZ.md).
Current project status is maintained in [docs/PROJECT_STATE.md](../PROJECT_STATE.md).
