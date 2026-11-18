# ADR-0008 — Background sync runtime and durable work ownership

**Date:** 2026-11-18  
**Status:** proposed with P0 implementation specification  
**Baseline:** main commit e380c58098878080b7d746f3cad7889effdc2fb6

## Context

PR #12 introduced the shared GitHub REST dispatcher, request telemetry and enforced API Monitor pause. The repository already has persistent api_queue_job, sync_outbox, sync_cursor and sync_conflict tables, but has no background worker. A production worker must avoid duplicating existing sources of truth and must not block UI operations.

## Decisions

1. One Tauri-owned worker/supervisor is started once and stopped explicitly on application exit. No detached worker and no worker-per-command.
2. The shared GitHub dispatcher is the only authenticated REST request path. Request class, principal and priority are explicit context; HTTP method alone does not determine request class.
3. api_queue_job is an execution envelope. sync_outbox is canonical mutation intent/state; sync_cursor is canonical incremental read progress.
4. Background quota is allocated per GitHub principal/resource/reset window. A reset window may not repeatedly re-grant its 10% allocation. A 20% observed-limit reserve is protected. Unknown/stale observations allow zero background work.
5. Ambiguous remote mutations are reconciled before retry. Outbox is never blindly replayed.
6. Cursor entity upsert and page-cursor advancement are committed atomically. A partial run cannot advance last_success_at/watermark.
7. No network I/O while holding AppState's outer mutex or a SQLite transaction/connection lock. Service/auth handles are cloned under lock and used after releasing it.
8. Worker scheduling only covers already tracked resources. Full history remains manual/cancellable.
9. Global API Monitor pause blocks authenticated REST. Background-sync enable/disable only changes scheduling and is not the same control.
10. PRs are reviewed and merged by the user; the implementation agent must not merge.

## Consequences

- Existing schemas/tables should be extended rather than replaced; any needed schema change is append-only.
- Queue, Outbox, cursor and provider-account state must be recoverable after abrupt exit.
- API Monitor and Sync settings can only show Active when runtime state proves the worker is alive.
- This ADR does not assert that the worker/budget/cursor work is already implemented; it records the intended architecture for the remaining P0.
