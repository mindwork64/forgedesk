# GitHub synchronization

## Current implementation

PR #12 is merged to main at `e380c58098878080b7d746f3cad7889effdc2fb6`.
Its exact CI run [44166546258](https://github.com/mindwork64/forgedesk/actions/runs/44166546258)
passed 7/7 jobs.

Implemented on main:

- GitHub App authentication and token storage.
- Repository discovery and bounded/manual Issue import.
- Manual bidirectional sync for already linked Issues: title, body, state and labels.
- Persisted Outbox, idempotent enqueue, ambiguous-outcome state machine,
  reconciliation and conflict records/resolution.
- Shared single-flight authenticated REST dispatcher, one-second mutation pacing,
  request telemetry, rate-limit observations and API Monitor Pause/Resume enforcement.

## What remains P0

1. Background scheduler and lifecycle-managed worker.
2. Shared REST + GraphQL dispatcher and secondary guard.
3. Adaptive per-account/per-resource budget accounting: 10% background allowance,
   20% reserve, zero allowance for stale/unknown observations.
4. Persistent incremental page cursors and watermarks for tracked repositories and
   linked Issues; active linked PR/check summary refresh.
5. Automatic dispatch/reconciliation of existing `update_issue` Outbox rows.
6. Truthful worker/queue/cursor/Outbox visibility in API Monitor and Sync settings.

## Queue ownership

- `sync_outbox`: write intents only.
- `api_queue_job`: scheduled/read jobs only.
- `sync_cursor`: page and watermark checkpoints.
- `api_request_log`: telemetry only.
- `api_budget_observation`: provider-observed quota.
- `api_control`: persisted pause.

Do not build duplicate queues or copy Outbox payloads into the API read queue. Never
run network I/O inside a SQLite transaction.

## Mutation safety

A queued update is not automatically safe to retry after a timeout. After a possible
send, mark the result ambiguous and reconcile remote state. Success requires every
field in the original payload to match. Mismatch creates/reuses a conflict and
requires user action. Failure to reconcile is not proof that the mutation failed.

## Detailed implementation contract

See [P0_REMAINDER_DEEPSEEK_TZ.md](../implementation/P0_REMAINDER_DEEPSEEK_TZ.md)
for the staged implementation plan, file map, acceptance tests and DeepSeek prompt.
