# Stage 5 report — GitHub Issue synchronization foundation

**Date:** 2026-11-15  
**Status:** PARTIAL — the local sync-safety foundation and GitHub App Device Flow
authentication are implemented and token acquisition/refresh were live-tested. Live
issue import/export and conflict UI remain pending. No remote write was attempted.  
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

## GitHub App authentication (delivered)

- Built-in public GitHub App Client ID with `FORGEDESK_GITHUB_CLIENT_ID` override.
- Device Flow: start/device code, polling interval/expiration handling, `slow_down`,
  denial/expiry errors; public UI shows code/URL and polls automatically.
- Token exchange validates `/user`, stores in OS keyring when available; memory-only
  fallback with explicit session warning when the keyring is unavailable.
- Refresh token flow implemented and tested; automatic refresh within 60s of expiry;
  disconnect removes the stored credential.
- **Live verification (manual test scenario):**
  - device code issued (`interval 5`, `expires_in 899`);
  - authorization completed;
  - token response: bearer, expiry 28800s, refresh token present, refresh expiry
    present, no scopes;
  - `/user` returned `mindwork64` (id 34349377), HTTP 200, rate 5000/4996;
  - refresh exchange returned HTTP 200 with new access + refresh tokens and 28800s
    expiry.
- Raw verification used in-memory tokens only; no token was written to the repo,
  docs, screenshots or logs.

## Not delivered (blocking live sync)

- Live REST client for repository discovery and read-only issue import exists;
  GraphQL, ETags, comment import, incremental cursor scheduling and field ownership
  enforcement remain open.
- Local→remote create/update/comment push and field ownership enforcement.
- Dispatcher worker consuming the outbox; reconciliation against remote objects.
- Sync conflict UI/resolution flow.
- Issue sync status/badges and manual sync controls.
- Tests against live GitHub or authenticated fixtures.

## Verification

| Check | Command | Result |
|---|---|---|
| Core tests | `cargo test -p forgedesk-core` | PASS — 52 (+1 ignored live) |
| Core clippy | `cargo clippy -p forgedesk-core --all-targets -- -D warnings` | PASS |
| Frontend typecheck/tests | `npm run typecheck`, `npm test` | PASS — 78 |
| Visual gates | `node scripts/visual-gate0.mjs` | PASS — 45 captures (auth + repository import) |
| Live read-only import | `GITHUB_TOKEN=$(gh auth token) cargo test ... --ignored` | PASS — cli/cli import, second run imported 0 and skipped links |
| Jenkins build 8 | Sync foundation `6d6bf52` | SUCCESS — 109 s, 46 core tests, 43 captures, 44 artifacts |
| Jenkins build 9 | Auth foundation `64f7469` | SUCCESS — 199 s, 49 core tests, 44 captures, 45 artifacts |
| Jenkins build 10 | GitHub discovery/import | Pending commit/mirror push |
| GitHub Actions | public repo `main` | SUCCESS — frontend, core Ubuntu/Windows, Tauri Linux/Windows, visual gates |
| Live GitHub sync | — | NOT RUN — auth/provider client absent |

## Next

1. Budget-aware REST/GraphQL client and rate-limit parser backed by the token vault.
2. Provider account persistence and repository discovery after auth.
3. Outbox dispatcher with reconciliation for ambiguous mutations.
4. Issue/label/comment pull + explicit local→remote write with field ownership.
5. Conflict UI and manual sync/status controls.

No mass synchronization is enabled until these exist and are tested.
