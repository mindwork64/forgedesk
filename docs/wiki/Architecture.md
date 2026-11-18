# Architecture

## Stack

- **Rust stable** core crate `forgedesk-core`: domain, SQLite repositories, services,
  Git wrapper, GitHub clients/sync and backups. No Tauri dependency.
- **Tauri 2** shell in `src-tauri`: typed IPC commands, lifecycle and window state.
- **Vue 3 + TypeScript + Vite** renderer: Pinia, Vue Router, vue-i18n, shared
  `Fd*` primitives and ForgeDesk design tokens.
- **SQLite** (bundled), WAL, foreign keys, serialized writes and append-only migrations.

## Layers

~~~text
Vue views/stores
   │ typed IPC
Tauri commands / lifecycle
   │ AppService
Domain + repositories + sync runtime
   │
SQLite / Git CLI / GitHub REST + GraphQL dispatcher
~~~

## GitHub request path

- Authenticated GitHub REST calls through `GitHubClient` use the shared dispatcher
  introduced in PR #12.
- The dispatcher currently provides one in-flight REST request, mutation pacing,
  telemetry, rate-limit observations and a persisted pause check.
- OAuth Device Flow/token exchange/refresh are handled separately from data API calls.
- GraphQL must join the same dispatcher/secondary guard before the P0 sync runtime
  is considered complete. Do not create independent HTTP paths.

## Sync persistence ownership

| Table | Responsibility |
|---|---|
| `sync_outbox` | Local-to-remote mutation intents and reconciliation state |
| `api_queue_job` | Durable scheduled/read jobs |
| `sync_cursor` | Per-resource page cursor, watermark and next due time |
| `api_request_log` | Request telemetry |
| `api_budget_observation` | Provider-reported resource budget |
| `api_control` | Persisted API pause control |

Do not create parallel queues or duplicate Outbox mutation payloads.

## Key principles

- Local-first; the app works offline without a backend.
- No fake UI: disconnected/empty/unsupported/incomplete states are explicit.
- Provider-neutral local domain; provider logic stays in adapters/services.
- Append-only migrations; never rewrite a published migration.
- No network I/O inside a SQLite transaction.
- Ambiguous remote mutations must be reconciled before retry; never blindly replay.
- API budget and worker state are per authenticated account/resource.
- PR #12 does not implement background scheduling, GraphQL dispatch, adaptive
  background quota enforcement or incremental cursor-based sync.

See ADR-0001…0007 and
[the P0 implementation spec](../implementation/P0_REMAINDER_DEEPSEEK_TZ.md).
