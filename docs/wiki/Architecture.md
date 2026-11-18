# Architecture

## Stack

- **Rust Stable** core crate `forgedesk-core`: domain, SQLite repositories, application services, Git and provider integration.
- **Tauri 2** shell in `src-tauri`: typed IPC commands, window lifecycle and application state.
- **Vue 3 + TypeScript + Vite** renderer: Pinia, Vue Router (hash), vue-i18n and shared `Fd*` primitives.
- **SQLite** (bundled), WAL, foreign keys, serialized writes, append-only migrations through `PRAGMA user_version`.

## Layers

```text
Vue views / stores
      │ typed IPC
Tauri commands / AppState
      │ AppService
Core domain + repositories
      │
SQLite / Git CLI / provider clients
      │
GitHub API dispatcher (REST today; GraphQL integration remains P0)
      │
GitHub REST / GraphQL endpoints
```

## GitHub request path

PR #12 (merged 2026-11-18) routes production `GitHubClient` REST requests through a process-wide single-flight gate. REST mutations are paced by at least one second. Response telemetry records operation/resource/class/status/latency/size and rate-limit headers without storing tokens or private payloads. The API Monitor pause flag is enforced before authenticated REST requests.

The current implementation is **REST-only**. OAuth Device Flow, token exchange and token refresh are separate authentication operations; authenticated `GET /user` uses the REST client. GraphQL transport, priority scheduling across API families, enforced background quota allocation and the adaptive scheduler remain P0 work.

## Sync persistence

- `remote_issue_link`: local ↔ remote Issue identity and field ownership.
- `sync_outbox`: durable outbound Issue updates with idempotency and ambiguous-outcome states.
- `sync_conflict`: explicit local/remote field conflicts.
- `sync_cursor`: persistent page cursor, watermark and due/success/error timestamps.
- `api_queue_job`: API Monitor queue/operation state model.
- `api_budget_observation` / `api_request_log`: provider budget observations and sanitized request telemetry.

The schema foundations exist, but automatic Outbox processing and cursor-driven incremental sync are not active. Manual sync must not be described as background sync.

## Key principles

- Local-first; the app works offline without a backend.
- No fake UI: disconnected/empty/unsupported states are explicit.
- Provider-neutral domain; GitHub adapters are isolated from issue/project storage.
- Append-only migrations; no destructive schema rewrite.
- No blind retry after ambiguous remote mutation; reconcile first.
- No network I/O while holding global AppState locks or SQLite write transactions.

See ADR-0001…0011 in `docs/decisions/` for the decision record.
