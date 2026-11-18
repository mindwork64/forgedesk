# ForgeDesk Wiki

ForgeDesk is a local-first desktop workspace for issues, projects, cycles, documents and provider-backed development workflows. It runs on Windows and Linux with Rust (Tauri 2), Vue 3 + TypeScript and SQLite.

## Start here

- [Architecture](Architecture)
- [Development](Development)
- [Project stages](Project-Stages)
- [API Budget and Sync](API-Budget-and-Sync)
- [Testing and release](Testing-and-Release)
- [Security and privacy](Security-and-Privacy)

## Current state — 2026-11-18

- Stage 1 — desktop foundation: complete.
- Stage 2 — issues/workflow/search/views: complete.
- Stage 3 — projects/planning/documents: partial.
- Stage 4 — Local Git and API Budget: partial. REST dispatcher, REST telemetry and enforced REST pause gate are merged; GraphQL dispatcher, priority scheduler and background budget enforcement remain open.
- Stage 5 — GitHub Issue synchronization: manual import and manual bidirectional field updates for linked Issues exist. Outbox/cursor foundations are persisted, but automatic Outbox and incremental sync workers remain open.
- Stage 6 — pull requests/checks/reviews: core list/detail and permission-gated actions exist; review-thread polish remains open.
- **PR #12:** merged as `e380c58098878080b7d746f3cad7889effdc2fb6`; CI run [44166546258](https://github.com/mindwork64/forgedesk/actions/runs/44166546258) passed 7/7 jobs.
- **Important:** REST dispatcher is Active; Background Sync is not implemented.

The source of truth is `docs/PROJECT_STATE.md`. See `docs/specs/P0_REMAINING_SYNC_SPEC.md` for the implementation handoff.
