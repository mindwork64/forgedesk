# ForgeDesk Wiki

## Current project state — 2026-11-18

ForgeDesk is a local-first, single-user desktop application built with Rust, Tauri 2, Vue 3/TypeScript and SQLite.

### Latest verified GitHub integration milestone

PR #12 is merged into main.

- Commit: e380c58098878080b7d746f3cad7889effdc2fb6
- Shared GitHub REST dispatcher is active.
- REST request telemetry and rate-limit observations are persisted.
- API Monitor Pause/Resume is functional and gates authenticated REST calls.
- OAuth Device Flow/token operations remain outside the REST pause gate.

CI for the exact PR #12 head passed 7/7 jobs before merge.

### Remaining P0

1. Application-owned background scheduler/worker.
2. Priority-aware request scheduling and adaptive API budget enforcement.
3. Automatic Outbox dispatch, retry and ambiguity reconciliation.
4. Persistent incremental cursors/watermarks.
5. Real worker/queue status in Sync settings and API Monitor.

The existing Outbox, cursor and queue tables are foundations; they are not evidence that the worker is complete.

### Canonical documentation

- Repository implementation specification: ../07_P0_BACKGROUND_SYNC_IMPLEMENTATION.md
- API budget and sync contract: ../../03_GITHUB_API_BUDGET_AND_SYNC.md
- Current project state: ../PROJECT_STATE.md
- Product gap audit: ../PRODUCT_GAPS_AUDIT.md
- Sync safety ADR: ../decisions/ADR-0006-sync-safety.md
- New background runtime ADR: ../decisions/ADR-0008-background-sync-runtime.md

See the dedicated P0-Background-Sync Wiki page for remaining implementation and acceptance criteria.
