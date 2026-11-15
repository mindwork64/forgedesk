# Stage 5 report — GitHub Issue synchronization foundation

**Date:** 2026-11-15  
**Status:** BLOCKED/PARTIAL — the local sync-safety foundation is implemented; live
GitHub import/export is blocked on GitHub App authentication and a live provider
client. No remote write was attempted.  
**Schema:** v5

## Delivered foundation

- Migration v5: `remote_issue_link`, `sync_cursor`, `sync_outbox`, `sync_conflict`.
- Remote link model: local issue ↔ provider/account/repository/remote id/number/URL,
  field ownership JSON, link state.
- Cursor model: per provider/account/resource/scope page cursor, watermark,
  last success/attempt, next due, last error, incomplete reason.
- Outbox state machine:
  `pending → dispatching → succeeded | failed_definite | outcome_unknown → reconciling
  → succeeded | retry_safe | needs_user_action`.
  Invalid transitions are rejected; ambiguous outcomes cannot jump directly to
  success; retries require an explicit safe/reconciled state.
- Idempotency-key uniqueness: enqueueing the same operation returns the existing job.
- Conflict model: field, local/remote values, remote timestamp, resolution and
  resolved timestamp.
- Tests: idempotent enqueue, attempts, ambiguous transition rejection, reconciliation
  path, link/cursor/conflict persistence and resolution (46 core tests total).
- No Tauri sync UI or provider calls: without auth, a Sync button would be fake.
- No tokens/secrets in the schema or logs.

## Not delivered (blocking live sync)

- GitHub App Device Flow, app client ID registration and secure token vault.
- Live REST/GraphQL client, ETags, pagination, incremental issue/label/comment pull.
- Local→remote create/update/comment push and field ownership enforcement.
- Dispatcher worker consuming the outbox; reconciliation against remote objects.
- Sync conflict UI/resolution flow.
- Issue sync status/badges and manual sync controls.
- Tests against live GitHub or authenticated fixtures.

## Verification

| Check | Command | Result |
|---|---|---|
| Core tests | `cargo test -p forgedesk-core` | PASS — 46 |
| Core clippy | `cargo clippy -p forgedesk-core --all-targets -- -D warnings` | PASS |
| Frontend typecheck/tests | `npm run typecheck`, `npm test` | PASS — 78 |
| Visual gates | `node scripts/visual-gate0.mjs` | PASS — 43 captures (no Stage 5 UI yet) |
| Jenkins | `ForgeDesk-CI` build 8 | Pending commit/mirror push |
| Live GitHub sync | — | NOT RUN — auth/provider client absent |

## Required next decision/action

Provide/configure a GitHub App client ID and choose secure token storage (OS
keychain/credential vault) before enabling Connect. Then implement:

1. Device Flow polling exactly per server responses (interval/expiration/slow_down).
2. Token vault + refresh/revocation; no token columns in SQLite.
3. Budget-aware REST/GraphQL client and rate-limit parser.
4. Outbox dispatcher with reconciliation for ambiguous mutations.
5. Issue/label/comment pull + explicit local→remote write with field ownership.
6. Conflict UI and manual sync/status controls.

Until then the app must show GitHub as not connected and must not offer a fake sync.
