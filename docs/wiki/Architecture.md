# Architecture

## Stack

- **Rust stable** core crate `forgedesk-core`: domain, SQLite repositories, services,
  backup, safe Git wrapper. No Tauri dependency.
- **Tauri 2** shell in `src-tauri`: typed IPC commands, window state, capabilities.
- **Vue 3 + TypeScript + Vite** renderer: Pinia, Vue Router (hash), vue-i18n, shared
  `Fd*` primitives and a small design-token system. No generic UI kit.
- **SQLite** (bundled), WAL, foreign keys, serialized writes, append-only migrations
  through `PRAGMA user_version`.

## Layers

```text
Vue views/stores
   │ typed IPC
Tauri commands
   │ AppService
Domain + repositories
   │
SQLite / Git CLI / (future provider client)
```

## Key principles

- Local-first; the app works offline without a backend.
- No fake UI: disconnected/empty/unsupported states are explicit.
- Provider-neutral domain; GitHub adapters are isolated from issue/project storage.
- Append-only migrations; no destructive schema rewrite.
- One issue workflow with human keys (`FD-<n>`), labels, relations, comments,
  activity and FTS search.
- Planning progress formula: completed / (total − canceled), excluding archived and
  deleted issues; initiatives aggregate completed/total across linked projects.

See ADR-0001…0005 in `docs/decisions/` for the full decision record.
