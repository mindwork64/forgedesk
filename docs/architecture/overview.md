# ForgeDesk architecture overview

**Status:** Stage 0 architecture lock. Stages 2+ add modules behind the same boundaries.

## Shape

ForgeDesk is one local desktop process. There is no backend service, no network
listener and no mandatory cloud dependency.

```text
┌────────────────────────────────────────────────────────────────────┐
│ Vue 3 + TypeScript renderer (Vite)                                 │
│ AppShell · views · command palette · i18n · design tokens          │
└───────────────────────────┬────────────────────────────────────────┘
                            │ typed Tauri IPC commands only
┌───────────────────────────▼────────────────────────────────────────┐
│ Tauri shell: src-tauri                                             │
│ command validation · error mapping · window lifecycle · app state  │
└───────────────────────────┬────────────────────────────────────────┘
                            │ Rust calls (no IPC, no HTTP from UI)
┌───────────────────────────▼────────────────────────────────────────┐
│ Core: crates/forgedesk-core                                        │
│ app services · domain rules · SQLite repositories · backups         │
│ (later: providers · sync · API budget · git · automation)          │
└───────────────────────────┬────────────────────────────────────────┘
                            │ SQLite (WAL)
                    app data directory
                    ├── forgedesk.db
                    └── backups/
```

The renderer never opens SQLite, runs shell commands or performs HTTP. Every
capability is a named, validated command. Destructive operations require an
explicit separate command and UI confirmation.

## Layer responsibilities

| Layer | Location | Responsibility |
|---|---|---|
| UI | `src/` | Rendering, local view state, keyboard interaction, i18n. No persistence or provider logic. |
| Shell | `src-tauri/` | Tauri lifecycle, window state, IPC validation, DTO/error mapping, state handle. |
| Application | `crates/forgedesk-core/src/app.rs` | Use cases: bootstrap, workspace, settings, backup/restore. |
| Domain | `crates/forgedesk-core/src/domain/` | Provider-neutral types and validation rules. No Tauri, HTTP or SQL. |
| Storage | `crates/forgedesk-core/src/db/` | Connections, PRAGMAs, migrations, repositories, transactions. |
| Backup | `crates/forgedesk-core/src/backup.rs` | Consistent snapshots, manifests, checksums, validation, pruning. |

Stage 2 adds the issue domain and repositories: workflow/status, labels, key
counter, issues, relations, comments, activity and FTS5 search. The filter engine
builds whitelisted parameterized SQL from a typed `IssueQuery`; UI never constructs
SQL. Later stages add `providers/`, `sync/`, `api_budget/`, `git/`, `search/` and
`automation/` inside the core, with the same dependency direction. Provider
adapters return provider-neutral DTOs; domain does not import them.

## Storage rules

- SQLite is the source of truth for the local model. WAL is enabled, foreign keys
  are enforced, writes are serialized through one guarded connection.
- Migrations are append-only and versioned through `PRAGMA user_version` plus a
  `schema_migration` audit table. Opening a database created by a newer schema is
  refused; it is never silently reset.
- Timestamps are UTC RFC 3339 with milliseconds. Local display is a UI concern.
- Local ids are UUID v7 strings. Human issue keys will use the workspace prefix
  (`FD-<n>`), introduced in Stage 2.
- Backups use `VACUUM INTO`, a SHA-256 manifest and SQLite integrity validation.
  Restore creates a safety snapshot before replacing the live file and reopens it
  transactionally; a failure rolls back to the previous database.

## Security boundary

- Tauri capability set is minimal (`core:default`); no fs, shell, http or dialog
  capability is granted to the renderer.
- The command surface contains no generic `execute_sql`, `run_shell` or
  `fetch_url`.
- Error payloads expose stable categories and sanitized messages. Raw SQL is
  never sent to the UI as a primary message; details are opt-in.
- Credentials will live in the OS credential store, never in SQLite or logs.

## Frontend architecture

- Vue 3 `<script setup>` + TypeScript, Vue Router, Pinia for cross-screen state.
- Design tokens are CSS custom properties in `src/styles/tokens.css`; components
  consume tokens only.
- `vue-i18n` with `en` and `ru` message catalogues from the first screen.
- Typed IPC facade in `src/lib/ipc.ts`; features never call `invoke` directly.
- UI Gate 0 is defined by `06_UI_UX_SPEC.md` §12 and implemented in Stage 1.

## Out of scope by design

No server, PostgreSQL/MySQL, Redis, Docker, Nginx, hosted telemetry, webhooks
relay, embedded Chromium or mandatory AI runtime.


## Current GitHub runtime after PR #12

Production `GitHubClient` REST requests pass through `github_dispatcher.rs`, which provides a process-wide single-flight gate, mutation pacing, REST telemetry and the persisted API Monitor pause gate. Authenticated `GET /user` uses this path; Device Flow/token exchange/refresh remain separate auth operations.

This is not yet the complete target in `03_GITHUB_API_BUDGET_AND_SYNC.md`: GraphQL dispatch, priority scheduling, enforced background quota, adaptive scheduler, automatic Outbox worker and cursor/watermark-driven incremental sync remain P0.


## Current full-product state — 2026-11-18

Main is e380c58098878080b7d746f3cad7889effdc2fb6, schema v11. PR #12 adds a shared process-wide GitHub REST single-flight gate, mutation pacing, REST telemetry/rate-limit classification and enforced REST pause. CI run 44166546258 passed 7/7 at tested head 405f3d4cf4080a49f154d7d29eea53bade4857b4.

The REST dispatcher is not the entire target API manager: GraphQL dispatch, priority scheduling, budget admission, adaptive polling, Outbox workers and cursor-driven sync remain. The rest of the unfinished product is tracked by docs/specs/PRODUCT_COMPLETION_SPEC.md.
