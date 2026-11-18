# P0 GitHub sync sub-specification

**This is a subsystem spec only. It is NOT the master plan for completing ForgeDesk.**  
The full project handoff is docs/specs/PRODUCT_COMPLETION_SPEC.md.

Use the detailed acceptance criteria below for the GitHub dispatcher/API budget/scheduler/Outbox/cursor track. It must be executed in the context of the whole product roadmap, not as a standalone project.

Baseline: main e380c58098878080b7d746f3cad7889effdc2fb6, schema v11, PR #12 merged 2026-11-18. PR #12 already delivered shared production REST single-flight, REST mutation pacing, REST telemetry/rate classification and enforced API Monitor pause. It did not deliver GraphQL dispatch or background sync.

---

## Delivered by PR #12

- Production `GitHubClient` REST requests pass through a process-wide single-flight gate.
- REST mutations are paced at least one second apart.
- REST telemetry stores operation/resource/class/status/latency/response size, Retry-After and rate headers using a token-free fingerprint.
- Primary and secondary REST rate-limit responses are classified separately.
- Persisted API Monitor pause is enforced before authenticated REST requests; Pause/Resume is functional in EN/RU.
- Authenticated `GET /user` uses the REST client. OAuth Device Flow/token exchange/refresh remain separate authentication operations.

CI run [44166546258](https://github.com/mindwork64/forgedesk/actions/runs/44166546258) passed 7/7 jobs at PR head `405f3d4cf4080a49f154d7d29eea53bade4857b4`.

## Exact remaining P0 scope

### A. Unified REST + GraphQL dispatcher

- Route every production GitHub REST and GraphQL call through one principal-aware dispatcher.
- Default max in-flight = 1. REST and GraphQL share the slot and secondary-limit guard.
- Keep >=1 second spacing between mutations across both API families.
- Implement real priorities: interactive write, interactive read, foreground refresh, background sync, manual full sync, reconciliation. Add fairness so background work cannot starve forever, while interactive work is never blocked by bulk sync.
- GraphQL query registry, variables, pagination, partial-data/error classification and actual `rateLimit.cost` recording. GraphQL HTTP 200 with `errors` is not automatically success.
- Pause blocks new REST and GraphQL API requests. Resume must not override provider reset/secondary pause.
- Do not hold global AppState locks or SQLite write transactions during network I/O, pacing or backoff.

### B. Enforced budgets and adaptive scheduler

Enforce background allowance per primary bucket:

`min(floor(10% * observed_limit), max(0, observed_remaining - round(20% * observed_limit)))`

- REST resource buckets and GraphQL points are independent; secondary pause is shared.
- Unknown/stale observations mean zero background allowance.
- Distinguish external account consumption from ForgeDesk's own request log; do not claim `limit - remaining` is only ForgeDesk usage.
- Respect Retry-After and reset. Secondary without Retry-After waits at least 60 seconds, then exponential backoff with jitter. Initial automatic retry cap: 3 safe reads/background operations.
- 401 stops provider queue pending auth; permission 403 is not a rate limit; 422 is not retried unchanged.
- Safe reads may use bounded retry. Mutation after possible send with no definitive response becomes `outcome_unknown` and must be reconciled, never blindly replayed.
- Local scheduler starts only after auth, not on disconnected startup. It stops on disconnect/401/app exit, observes pause/offline/budget, persists due state and restores after restart.
- Initial intervals: active linked Issues/context 10–15 min; selected repository metadata 30–60 min; inactive repositories hours; historical sync manual only. Use jitter and central config.
- Do not poll PR/check resources unless a durable cache/cursor exists; unsupported resources remain explicitly unscheduled.

### C. Automatic Outbox worker

- `sync_outbox` remains the durable source of outbound mutations. Start with the already supported linked-Issue `update_issue` operation.
- Atomically claim a job and mark `dispatching` before sending. Prevent two workers claiming the same job. Preserve ordering per Issue and prevent bounded-query starvation.
- On restart, stale `dispatching` becomes `outcome_unknown`; reconcile first.
- State flow remains: `pending → dispatching → succeeded | failed_definite | outcome_unknown`; `outcome_unknown → reconciling → succeeded | retry_safe | needs_user_action`; `retry_safe → pending`.
- Classify definite validation/permission failures separately from ambiguous network outcomes.
- If remote intended fields match, mark success. If unchanged and safe retry is proven, mark retry_safe. If remote changed concurrently or result remains ambiguous, persist conflict/needs-user-action.
- Persist next retry time/backoff across restart. Unsupported operations are visible, never silently skipped or sent with unknown payload.
- Never auto-retry a write simply because a GET reconciliation failed due to network/rate limit/auth.

### D. Incremental cursor/watermark sync

- Use existing `sync_cursor` per provider/account/repository/resource/query contract.
- Keep page cursor (opaque continuation) distinct from time watermark.
- Use updated_at ordering and an overlap window; upsert by remote opaque ID.
- Fetch each page outside a DB write transaction. Apply page rows and advance cursor/watermark atomically in one SQLite transaction.
- Failure/partial apply does not advance last_success_at. Repeated pages must be logically idempotent.
- Changing query shape/filter/sort invalidates or versions the cursor.
- Detect pagination loops/non-advancing cursors. Filter PRs from Issue imports.
- A missing/inaccessible/deleted remote object must not automatically delete the local Issue; retain tombstone/incomplete state and reason.
- Full historical sync remains explicit, cancellable and resumable; forecasted cost is clearly marked estimate.

## Existing files and tables to inspect/reuse

Code: `github_dispatcher.rs`, `github_client.rs`, `github_sync.rs`, `app.rs`, `github_auth.rs`, `domain/api_budget.rs`, `domain/sync.rs`, `db/repositories/api_budget.rs`, `db/repositories/sync.rs`, `db/migrations.rs`, `src-tauri/src/state.rs`, `src-tauri/src/commands/mod.rs`, `src/features/api-monitor/ApiMonitorView.vue`, `src/stores/apiMonitor.ts`, `src/lib/ipc.ts`, `src/types/ipc.ts`, `src/i18n/en.ts`, `src/i18n/ru.ts`.

Reuse `api_control`, `api_budget_observation`, `api_request_log`, `api_queue_job`, `sync_outbox`, `sync_cursor`, `sync_conflict`. Keep responsibilities distinct: cursor=schedule/checkpoint; outbox=durable writes; api_queue_job=observable API execution queue state without raw private payload. Avoid duplicate queue implementations.

Schema v11 is current. Never edit migrations 0001–0011; add 0012+ append-only with defaults and v11 upgrade/data-preservation/FK/integrity tests if needed.

## Test requirements

Use deterministic fake clock + offline mock HTTP fixtures; no real credentials or live remote mutations in CI.

- Dispatcher: shared REST/GraphQL single-flight, priority/fairness, cross-family mutation pacing, pause/resume, cancellation/shutdown, token redaction, no global locks during I/O.
- Budget: formula boundary cases, stale/unknown block, REST buckets separated, GraphQL cost points separate, secondary shared, Retry-After/reset, permission 403 distinction, GraphQL HTTP 200 errors, bounded retries.
- Scheduler: disconnected startup produces no request; starts after auth; due/backoff/jitter; budget/pause/offline/disconnect behavior; restart restoration; only one worker; graceful shutdown.
- Outbox: duplicate enqueue idempotency, atomic claim, interruption recovery, remote already updated→success, proven safe retry, concurrent edit→conflict, failed reconciliation deferred, unsupported operation visible, no starvation.
- Cursor: restart persistence, atomic apply/checkpoint, failure doesn't advance, repeated page no duplicates, overlap boundary, query contract invalidation, Link/endCursor loop protection, inaccessible remote objects do not delete local task, PR excluded.
- Migration: v11→new schema preserves issue/project/link/outbox/cursor/conflict/API observation data, FK and integrity pass.
- Full CI: fmt, Core clippy/tests on Ubuntu/Windows, Frontend typecheck/tests/build, optional AI tests/clippy, Tauri Linux/Windows compile, UI visual gates.

Manual Windows/Linux interactive smoke tests with real account must be reported separately from CI.

## UI and documentation

API Monitor must show real worker state (stopped/disconnected/running/paused/backoff/error), next due, queue counts, background allowance/reserve/usage, separate REST/GraphQL primary observations and Outbox state counts. Never show Background Sync Active until a real worker has started. Localize EN/RU and preserve UI Gate assertions. Update PROJECT_STATE, product gaps audit, changelog, architecture/stage reports and wiki mirror.

## Token-efficiency requirements

Follow `04_TOKEN_EFFICIENCY_RULES.md` on every DeepSeek call:

- Keep the existing warm prompt from `05_DEEPSEEK_PROMPTS.md` unchanged and first in the stable cache prefix.
- Read PROJECT_STATE + only relevant spec sections/ADR/3–8 directly related files; no repeated repository-wide scan.
- Keep static prompt prefix and document order stable; append changing state/logs after it.
- Use minimal unified diffs; don't reprint whole files for small edits.
- Filter successful build logs; pass exit code + focused error excerpts only.
- Don't repeat successful tests unless code/environment changed, but never skip required stage/full CI.
- Keep PROJECT_STATE compact (target 1,500–2,000 tokens) and write concise stage reports.
- Track usage/cache hit/cache miss/input/output/reasoning/duration per stage if available; don't send secrets or private code to external analytics.
- Save tokens on repetition, not on validation/security/tests.

## Stage order

A. GraphQL + shared dispatcher completion.  
B. Enforced budget + scheduler runtime.  
C. Durable automatic Outbox worker and reconciliation.  
D. Incremental Issue cursor/watermark flow.  
E. API Monitor/Settings integration, migration/CI hardening, docs/wiki and manual smoke checklist.

Each stage must build and have target tests before proceeding. Final Definition of Done: every requirement above passes; exact tested PR head and CI URL are reported; unsupported live/native tests remain explicitly unverified; no PR merge/release without user approval.
