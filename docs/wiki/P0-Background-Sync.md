# P0 — Background Sync

**Status:** pending implementation. PR #12 has completed the shared REST dispatcher/pause foundation only.

## Already implemented

- Shared authenticated GitHub REST gate in github_dispatcher.rs.
- Request telemetry, request fingerprint and REST rate-limit observations.
- API Monitor Pause/Resume persisted in api_control and enforced before authenticated REST calls.
- Existing durable sync_outbox, sync_cursor, sync_conflict and api_queue_job schemas.
- Manual bidirectional sync for already linked GitHub Issues (title/body/status/labels), with conflict resolution.

PR #12 merge commit: e380c58098878080b7d746f3cad7889effdc2fb6. CI for PR head: 7/7 successful.

## Remaining implementation

### A. Worker lifecycle and dispatcher context
One Tauri-owned worker; no detached threads; no AppState mutex or SQLite transaction across HTTP. Make request classes and priority effective. One in-flight request per GitHub principal.

### B. Durable queue and adaptive budget
Use existing api_queue_job as execution envelope; Outbox and cursor remain canonical sources. Add an append-only migration if required. Enforce 10% background allowance per reset window and 20% reserve. Unknown/stale observations mean zero background allowance. Scope by provider account. Respect primary/secondary limits and manual pause.

### C. Automatic Outbox
Automatically dispatch existing update_issue Outbox rows for already linked GitHub Issues. Reuse manual dispatch implementation. Never retry outcome_unknown until reconciliation GET completes. Do not add comments or new remote mutation types in this P0.

### D. Incremental sync
Use sync_cursor in production. Persist opaque page continuation and a separate watermark. Apply remote entities and advance cursor in one SQLite transaction. Use a 2–5 minute overlap window. Partial/failed runs must not advance last_success_at.

### E. UI and acceptance
API Monitor and Settings → Sync show real worker, queue, due time, budget and reconciliation state. RU/EN. No placeholder Active status.

## Defaults

- One in-flight request per principal.
- At least 1 second between mutative REST requests.
- Active PR/check summaries: 10–15 min if tracked and budget permits.
- Active Issues: 30–60 min if tracked and budget permits.
- Inactive repository metadata: 4–8 hours.
- Historical full sync: manual and cancellable only.
- Background quota: min(10% of observed limit, remaining above 20% safety reserve).
- Stale/unknown budget: background allowance 0.
- Secondary limit without Retry-After: wait at least 60 seconds, then bounded backoff.
- Safe read retries: maximum 3 automatic attempts.
- Ambiguous mutation: reconcile; never blind replay.

## Explicit exclusions

No GitLab/Gitea live sync, comments, automatic publishing of all local Issues, new GraphQL client, webhooks, cloud/backend service or full historical startup scan.

## Canonical source

See repository root file 07_P0_BACKGROUND_SYNC_IMPLEMENTATION.md for architecture, migration design, mandatory tests, stage plan and DeepSeek starter prompt.
