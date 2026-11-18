# ForgeDesk — P0 remainder: GitHub sync runtime and background worker

**Document type:** implementation specification / handoff to DeepSeek API  
**Prepared:** 2026-11-18  
**Target repository:** https://github.com/mindwork64/forgedesk  
**Implementation base:** \`main\` at \`e380c58098878080b7d746f3cad7889effdc2fb6\` (PR #12 squash merge)  
**Priority:** P0 — required to finish the current GitHub synchronization foundation  
**Status:** ready for implementation; this document does not claim that the functionality is already implemented

---

## 1. Mission and definition of done

Implement the remaining P0 GitHub synchronization runtime on top of the merged REST dispatcher from PR #12. ForgeDesk must keep selected GitHub repositories/issues and linked pull-request context incrementally synchronized while the desktop application is running, dispatch already queued local Issue updates safely, recover after crashes/restarts, respect GitHub primary/secondary limits, and report truthful runtime state in API Monitor and Sync settings.

This is an implementation task, not a request for another architecture proposal. Inspect the current source, choose the smallest safe design consistent with this specification, implement it, add tests, and create reviewable pull request(s).

P0 is complete only when all of the following work end-to-end:

1. A lifecycle-managed background scheduler runs only while ForgeDesk is running and GitHub is connected.
2. Persistent read/sync jobs use the existing API queue and cursor model; no second general-purpose job queue is created.
3. REST and GraphQL requests share one dispatcher, one in-flight policy, the pause/rate-limit guard, priority scheduling and telemetry. REST dispatcher work from PR #12 must be preserved, not replaced.
4. Adaptive quota allocation actually prevents background work from consuming the protected reserve or running with unknown/stale budget observations.
5. Existing Outbox mutations are dispatched automatically and safely, with crash recovery and reconciliation. No mutation is blindly retried after an ambiguous result.
6. Repository/Issue incremental sync uses persisted cursors and watermarks and can resume after app restart without losing or skipping changes.
7. API Monitor / Sync settings accurately expose worker, queue, cursor, Outbox, pause, rate-limit and next-run state in English and Russian.
8. Unit, integration, migration, fixture and UI tests cover the acceptance criteria below. CI must pass.

Do not label the feature complete merely because a timer thread starts or because manual sync still works.

## 2. Verified baseline — do not re-implement

The following facts were checked against \`main\` at the implementation base SHA above.

### 2.1 PR #12 is already merged

PR #12: https://github.com/mindwork64/forgedesk/pull/12  
Merge commit: \`e380c58098878080b7d746f3cad7889effdc2fb6\`  
Tested PR head: \`405f3d4cf4080a49f154d7d29eea53bade4857b4\`  
GitHub Actions run: https://github.com/mindwork64/forgedesk/actions/runs/44166546258 — **7/7 jobs passed**.

It added:

- \`crates/forgedesk-core/src/github_dispatcher.rs\`: process-wide single-flight REST request gate, mutation pacing, request logging, rate-limit observation and a persisted pause/secondary/primary authorization check.
- REST calls made through \`GitHubClient\` in \`crates/forgedesk-core/src/github_client.rs\` pass through that dispatcher.
- Request classes already exist: \`interactive_read\`, \`interactive_write\`, \`foreground_refresh\`, \`background_sync\`, \`manual_full_sync\`, \`reconciliation\`. However, the currently wired GitHub client mostly assigns interactive defaults; background scheduling and class-aware quota enforcement are not complete.
- \`api_control.paused\` is enforced for authenticated REST calls through \`GitHubClient\`; API Monitor has a working Pause/Resume control.
- API request telemetry stores operation/resource/class, fingerprint, status, latency, payload size and rate headers without storing authorization tokens.
- The authenticated \`GET /user\` identity check uses the shared REST client. OAuth Device Flow and token exchange/refresh remain separate authentication operations by design.

Do not remove or regress this functionality. Extend its contracts to cover the work below.

### 2.2 Existing persistence is the foundation

Current schema version is **11**. Migrations are append-only in \`crates/forgedesk-core/src/db/migrations.rs\`.

Relevant existing tables:

- \`api_budget_observation\`, \`api_request_log\`, \`api_queue_job\`, \`api_control\` — migration \`0004_git_budget.sql\`.
- \`remote_issue_link\`, \`sync_cursor\`, \`sync_outbox\`, \`sync_conflict\` — migration \`0005_sync.sql\`.
- \`pull_request_link\` — migration \`0006_pull_links.sql\`.

Relevant existing code:

- \`crates/forgedesk-core/src/domain/api_budget.rs\`
- \`crates/forgedesk-core/src/db/repositories/api_budget.rs\`
- \`crates/forgedesk-core/src/domain/sync.rs\`
- \`crates/forgedesk-core/src/db/repositories/sync.rs\`
- \`crates/forgedesk-core/src/github_sync.rs\`
- \`crates/forgedesk-core/src/github_dispatcher.rs\`
- \`crates/forgedesk-core/src/github_client.rs\`
- \`crates/forgedesk-core/src/app.rs\`
- \`src-tauri/src/state.rs\`
- \`src-tauri/src/lib.rs\`
- \`src-tauri/src/commands/mod.rs\`
- \`src/features/api-monitor/ApiMonitorView.vue\`
- \`src/features/settings\` (inspect the existing Sync settings view/store before changing it)
- \`src/lib/ipc.ts\`, \`src/types/ipc.ts\`
- \`src/i18n/en.ts\`, \`src/i18n/ru.ts\`

### 2.3 Current limitations confirmed in source

- \`sync_cursor\` and repository functions \`upsert_cursor/get_cursor\` exist, but current Issue import is a bounded manual REST import using page numbers. The cursor/watermark is not the active incremental sync checkpoint.
- \`sync_outbox\` is persisted and idempotent. Local edits to already linked GitHub Issues can enqueue \`update_issue\` operations. The current dispatcher is exposed as a manual Tauri command and processes only \`update_issue\`; it is not a background worker.
- Current Outbox logic already handles \`dispatching\` after restart as \`outcome_unknown\`, performs a GET reconciliation for remote Issue state and creates conflicts rather than silently overwriting a mismatch. Preserve and strengthen this safety behavior.
- The current \`api_queue_job\` is a persistence foundation and API Monitor data source; there is no lifecycle-managed scheduler consuming it.
- \`RequestClass\` contains background classes, but the live REST request call path does not yet receive the appropriate class from each scheduler/manual/background operation.
- The active code tree has no dedicated GitHub GraphQL client module. Existing GitHub features are primarily REST-backed. The accepted project API specification (see \`03_GITHUB_API_BUDGET_AND_SYNC.md\`) requires GraphQL-first where appropriate, REST fallback for operations/fields not suitably exposed, and a common dispatcher/secondary guard.
- \`AppState\` owns service/auth state behind a mutex. The worker must not hold the global Tauri state lock during long network operations; use short lock scopes and the thread-safe SQLite boundary.
- The current API Monitor displays a limited queue projection and request telemetry. It must be upgraded to show actual background worker and Outbox state, not synthetic “active” indicators.

If the source has moved since this base commit, re-check the current branch and adapt these references. Do not overwrite changes made after this baseline.

## 3. Locked architecture and queue ownership

### 3.1 One source of truth per type of work

Use the existing persistence model; do not add a second generic queue or duplicate payloads.

| Existing table | Canonical responsibility | Must not become |
|---|---|---|
| \`sync_outbox\` | Durable local-to-GitHub mutation intents and their reconciliation state | A mirror of read/poll jobs |
| \`api_queue_job\` | Durable scheduled/read work: discovery refresh, repository/Issue/PR sync, manual bulk read sync | A second copy of Outbox mutations |
| \`sync_cursor\` | Checkpoint and freshness state per provider/account/resource/scope | A queue or an unbounded event log |
| \`api_request_log\` | Completed/failed request telemetry | A queue or a source of desired work |
| \`api_budget_observation\` | Latest observed provider budget per authenticated principal and API bucket | A synthetic estimate of all usage by the user |
| \`api_control\` | User pause control and persisted pause reason | A transient in-memory flag |

API Monitor may render a unified *read-only projection* of \`api_queue_job\` and eligible \`sync_outbox\` rows. Do not duplicate Outbox jobs into \`api_queue_job\` just to make the UI easier.

### 3.2 Shared request dispatcher

Retain and extend \`github_dispatcher.rs\` into the common REST + GraphQL dispatcher. A request must carry at least:

- authenticated provider/account identity (never the token);
- API family and provider resource/bucket;
- operation code/name;
- request class and priority;
- estimated cost;
- read vs mutation classification;
- retry/reconciliation classification;
- optional deadline/cancellation for user-initiated work;
- sanitized request fingerprint.

All GitHub data requests in feature modules must go through this path. Do not create independent \`reqwest::Client\` request paths for Issues, repositories, PRs, reviews, checks, or GraphQL.

OAuth Device Flow/token-exchange/token-refresh protocol calls remain in the authentication subsystem as documented; do not accidentally route credentials through telemetry or persist raw credentials. The authenticated account identity request remains in the shared data API path.

**Default concurrency:** one in-flight GitHub data request per authenticated principal. REST and GraphQL share the same slot and secondary-limit guard.

**Priority order (lower number = higher priority):**

| Priority | Class | Examples |
|---:|---|---|
| P0 | \`interactive_write\` | User explicitly saves/executes an Issue or PR mutation |
| P1 | \`interactive_read\` | User opens a view or manually refreshes the visible entity |
| P2 | \`foreground_refresh\` | Current project, active Issue, selected repository or linked PR context |
| P3 | \`background_sync\` | Periodic incremental sync of tracked repositories/entities |
| P4 | \`manual_full_sync\` | User-started historical/backfill read sync |

A lower priority request must never delay an already queued higher-priority user operation indefinitely. Use FIFO sequence within a priority. Do not pre-empt an in-flight network request. Coalesce duplicate pending reads by principal + operation + scope + query fingerprint. Never coalesce mutations.

Mutation pacing remains at least one second between writes, including when alternating REST and GraphQL. Do not add an unconditional one-second delay to every read. If secondary-limit signals appear, the shared dispatcher pauses all relevant requests according to the provider response.

### 3.3 REST and GraphQL

- GraphQL is preferred for list/summary/detail sync where it provides the needed fields with a lower request count and stable cursor semantics.
- REST remains valid for endpoints/operations not suitably exposed by GraphQL (including existing PR mutation/feature paths where already implemented).
- Both transports must share: authorization gate, priority scheduling, single-flight, mutation pacing, cancellation/deadline semantics, backoff, budget accounting, redacted telemetry and pause behavior.
- GraphQL HTTP 200 with a rate-limit error in \`errors\` is a rate-limit event, not success. Parse GraphQL \`rateLimit.cost\`, \`remaining\`, \`resetAt\` when returned.
- Persist REST \`x-ratelimit-*\`, \`retry-after\`, \`ETag\`/conditional metadata where used. Record GraphQL actual cost; do not count a query as cost 1 when the response gives a different actual cost.
- Query documents must be constants/files with variables, minimal field sets, bounded page sizes and tests. Do not generate arbitrary unbounded GraphQL documents from UI values.
- GraphQL page size must not exceed 100; start with 25 for detailed lists and 50 for light summaries. Respect the existing 10,000 predicted-node internal query ceiling from \`03_GITHUB_API_BUDGET_AND_SYNC.md\`.
- Do not rewrite every existing REST-backed screen to GraphQL as incidental cleanup. Convert the sync operations needed for P0 where GraphQL is suitable; preserve REST for the rest and route both through the dispatcher.

### 3.4 No network inside a SQLite transaction

Every sync operation uses three distinct phases:

1. Read the persisted cursor/desired work and prepare a bounded request.
2. Perform network requests through the dispatcher with no SQLite transaction open.
3. Apply each completed page and advance its cursor in one short transaction.

If a transaction fails, the page checkpoint must not advance. If the process exits after the remote response but before the transaction commits, the page must be safe to fetch/apply again by provider opaque ID/upsert semantics.

## 4. Background worker lifecycle

### 4.1 Startup, shutdown, and thread safety

Create a lifecycle-managed sync runtime owned by \`AppState\` (or an equivalent Tauri-managed component). It must:

- start once after the DB/service is initialized;
- begin in an idle state and not issue bulk requests on application launch;
- load/validate connected GitHub account state before scheduling;
- operate only while the desktop process is running;
- stop accepting new work during shutdown;
- allow a safe in-flight request to finish within the configured network timeout, or persist an unknown/incomplete outcome;
- persist all due jobs/cursors before exit;
- join/stop the worker deterministically and idempotently;
- survive repeated start/stop, window-close/exit and auth reconnect/disconnect without duplicate workers.

Do not hold \`AppState\`'s global mutex across a network request. Do not hold a SQLite transaction, DB connection mutex or service lock while waiting for a timer/backoff/network response. Clone the minimum required service/token state under a short lock, release it, then perform the bounded operation. Ensure \`AppService\`/database access remains serialized and thread-safe.

Use an interruptible wait (channel/Condvar/stop token) rather than an uninterruptible sleep so Pause, Resume, Disconnect and Shutdown can wake the worker immediately.

### 4.2 Triggers

- Application start: render local cached data first; enqueue no full-history sync.
- Successful GitHub connection/reconnection: validate identity, refresh budget observations through normal API requests and schedule a small bounded discovery pass.
- App becomes foreground: refresh only stale active context if priority/budget permit.
- Opening an Issue/PR/repository: enqueue or coalesce a P1/P2 read for that entity.
- Manual refresh: P1; never wait behind P3/P4 work.
- Network unavailable: delay background work with bounded exponential backoff and jitter. Do not spin.
- Network restored: wake the scheduler and resume eligible jobs after checking pause/budget.
- API Monitor Pause: stop dispatching new GitHub data requests; keep jobs persisted as paused/delayed. Resume wakes the dispatcher.
- Auth expires/revoked/disconnects: stop provider jobs, retain local queue/cursor state and expose reconnect-required; do not repeatedly call GitHub using an invalid token.
- App shutdown: graceful stop and persist state. No background sync while ForgeDesk is closed.

### 4.3 Scheduling tiers and initial defaults

Centralize all tuning values; do not scatter interval literals across modules. Use bounded jitter to avoid periodic request bursts.

| Work | Initial default | Conditions |
|---|---|---|
| Active linked PR/check summary | 10–15 minutes | PR is linked to an active local Issue or currently open; budget allows |
| Tracked repository + linked Issue incremental refresh | 30–60 minutes | Repo is explicitly discovered/selected/tracked; budget allows |
| Inactive tracked repository metadata | Several hours | No active UI context; budget allows |
| Full historical import/backfill | Manual only | Explicit user action, visible estimate and cancellation |

These are defaults, not hard guarantees. Scheduler increases intervals when the app is idle, backlog is low, budget is low/unknown, latency/error rate rises or secondary limit is active.

Do not poll closed/archived historical data on a tight loop. Do not sync every accessible GitHub repository automatically. Only repositories the user has discovered/selected/tracked inside ForgeDesk are eligible.

## 5. Adaptive API budget and rate-limit enforcement

Implement the policy already specified in \`03_GITHUB_API_BUDGET_AND_SYNC.md\` and \`domain/api_budget.rs\`.

### 5.1 Formula

~~~text
background_allowance =
  min( floor(0.10 * observed_limit),
       max(0, observed_remaining - ceil(0.20 * observed_limit)) )
~~~

- The 10% cap and 20% reserve are initial defaults and must be centralized/configurable within safe bounds.
- Unknown or stale \`limit/remaining/reset\` => background allowance is zero.
- User-initiated P0/P1/P2 work may use available provider budget beyond the background allowance, but must never bypass actual provider primary or secondary limits.
- A manual P4 full sync must be explicit, estimated and cancellable; it does not silently consume the protected background reserve.
- Primary budget is tracked by the actual provider bucket/resource. Do not assume all REST traffic is \`core\`; use \`x-ratelimit-resource\` when present and correct endpoint fallbacks otherwise.
- REST and GraphQL primary buckets must be accounted independently when GitHub reports separate buckets; the secondary guard and single-flight slot are shared.

### 5.2 Usage accounting and freshness

- An observation is authoritative only for the authenticated principal and bucket that produced it.
- Store \`provider_account_id\`/stable account identity on new observations and request rows when available. Never combine observations from different GitHub accounts.
- Background usage since the latest observation is calculated from ForgeDesk's own request log using actual/estimated cost and request class. Do not infer that \`limit - remaining\` is all ForgeDesk usage; other applications may consume the same account quota.
- Treat a missing cost conservatively; do not grant extra background capacity because telemetry was incomplete.
- Apply successful REST request cost conservatively; a valid 304 conditional response must not be charged as a normal primary read if GitHub documents it as free. Keep it in telemetry with \`not_modified\`.
- GraphQL uses the response's actual query cost when available.
- A stale observation must not authorize background work. Pick a documented freshness threshold tied to reset window/observation age and test exact boundary cases.
- Replace stale forecast with the next real provider observation; expose estimates as estimates in UI.

### 5.3 Primary/secondary rate limits

- Primary exhausted (\`remaining = 0\`): pause only the affected bucket until reset; do not probe repeatedly before reset.
- Secondary limit: pause shared dispatch according to \`Retry-After\`; if absent, wait at least 60 seconds, then bounded exponential backoff with jitter. Do not allow a separate REST/GraphQL path to bypass this.
- \`403\` is not automatically a rate limit: distinguish secondary/primary from permission/authorization failures.
- GraphQL \`200 + errors\` may still be rate-limited; parse body errors.
- Initial automatic retry ceiling for safe reads/background jobs: 3 attempts. After that, delay or require user action and show the reason.
- Reads may retry within bounds. Mutations must reconcile first if outcome could be ambiguous.
- \`401\`: stop provider queue and require reconnect.
- \`403\` permission error / \`404\` private-resource access ambiguity / \`422\` validation: classify accurately; no infinite retry.
- \`502/503/504\` can retry safe reads with backoff; mutation moves to reconciliation if it may have reached GitHub.
- Respect rate-limit headers and reset UTC epoch exactly; display local time in UI.

### 5.4 Pause behavior

Extend the existing API Monitor pause gate so Pause blocks all authenticated GitHub *data API* requests through both REST and GraphQL dispatchers, including scheduler and reconciliation. OAuth Device Flow/refresh remains separate as currently documented.

Pause must not delete jobs, reset cursors, mark work as succeeded, or corrupt Outbox states. It prevents new requests and reschedules/waits eligible work. An already in-flight network request is not forcibly terminated solely because the user pressed Pause; after it returns, no next request may begin until Resume.

## 6. Persistent queue and incremental cursor design

### 6.1 \`api_queue_job\` — scheduled/read jobs

Use \`api_queue_job\` for durable sync/read jobs (repository discovery refresh, repository metadata, Issue incremental page, PR/check summary, manual full read sync). Reuse existing fields \`priority\`, \`request_class\`, \`state\`, \`attempts\`, \`estimated_cost\`, \`fingerprint\`.

Add an append-only migration only for missing capabilities. Likely needed fields include:

- stable dedupe key / logical job identity;
- bounded payload/scope JSON containing only non-secret selectors;
- \`not_before_utc\` (retry/due time);
- \`lease_until_utc\` or equivalent crash-safe claim expiry;
- \`started_at_utc\` / \`completed_at_utc\` if useful for UI/recovery;
- cancellation request/state where manual bulk work needs cancellation.

Add indexes/uniqueness that prevent duplicate due jobs for the same principal/resource/scope/query fingerprint. A claim must be atomic (transactional state transition or compare-and-set); two worker wakeups must not run the same row concurrently.

Persisted job payloads must not contain OAuth access/refresh/device tokens, headers, raw private issue bodies, diffs or comments. Use opaque IDs, repository scope and query metadata only.

Do not invent \`sync_job\`, \`background_queue\` or another parallel queue if the existing \`api_queue_job\` can represent this work. If a schema limitation truly requires a separate table, document the concrete limitation in an ADR first.

### 6.2 \`sync_outbox\` remains the write-intent source

Do not copy Outbox payloads into \`api_queue_job\`. Extend \`sync_outbox\` append-only only for fields required by safe automatic dispatch: e.g. \`next_attempt_at\`, \`lease_until\`, last attempt metadata or a typed failure category.

The Outbox state machine remains authoritative:

~~~text
pending
  -> dispatching
  -> succeeded | failed_definite | outcome_unknown
outcome_unknown -> reconciling
reconciling -> succeeded | retry_safe | needs_user_action
retry_safe -> pending
needs_user_action -> pending (only after explicit user resolution) | cancelled
~~~

Maintain closed transition validation in \`OutboxState::can_transition_to\`. Do not introduce arbitrary string transitions.

### 6.3 Cursor key and semantics

The logical cursor key is:

~~~text
(provider, account_id, resource, scope, query_fingerprint)
~~~

- \`scope\` must identify a stable repository/entity scope, e.g. GitHub repository opaque ID or canonical \`owner/repo\` bound to the account.
- \`resource\` distinguishes repository metadata, issue list, pull-request list/check summary, etc.
- \`query_fingerprint\` changes when filters, sort order, API version or query shape changes. Never reuse a cursor for a changed query.
- Keep page cursor and time watermark separate: they represent different facts.
- Page cursor is opaque. For REST, parse the response \`Link\` header; do not synthesize next URLs from page numbers if the endpoint gives a next link. Validate that the next URL stays on the expected GitHub API host/path before following it.
- For GraphQL, persist \`endCursor\` with \`hasNextPage\`.
- Preserve \`last_success_at\`, \`last_attempt_at\`, \`next_due_at\`, \`last_error\` and \`incomplete_reason\`.

### 6.4 Incremental Issue sync

Implement incremental refresh of already tracked repositories and their GitHub Issues:

1. Read account/repository cursor and previous watermark.
2. Set a stable query using a deterministic sort and filter, ideally updated-time ascending with an overlap window (initial default: 5 minutes; centralize/configure).
3. Fetch one bounded page through the shared dispatcher.
4. Apply the entire valid page with upsert by GitHub opaque Issue ID. Skip pull-request-shaped objects from the Issue importer as current code does.
5. In the same short SQLite transaction, persist the next page cursor and attempt metadata. Advance the durable successful watermark only after the complete incremental window is applied.
6. When more pages exist, persist the page checkpoint and enqueue/resume the next page. A restart continues without starting historical sync from page one.
7. When the window completes, clear the page cursor, advance watermark to the maximum safely observed update boundary (with overlap retained), set \`last_success_at\`, calculate \`next_due_at\` and clear transient error/incomplete state.
8. On any network, parse, validation or DB failure, retain the last successful watermark and the recoverable page checkpoint. Set \`last_error\`/\`incomplete_reason\`; do not report full success.
9. Use overlap to tolerate equal timestamps, delayed indexing and page-boundary races. Upsert/dedupe prevents duplicates.
10. Remote deletion/absence is not an instruction to hard-delete the local Issue. Retain links/tombstone/sync state and surface a safe status.

The existing five-page bounded manual import can remain as a user-started discovery/backfill path, but incremental scheduled sync must have its own persisted cursor semantics and must not masquerade as a complete full import.

### 6.5 PR/check refresh

Use existing \`pull_request_link\`, existing PR list/detail/check models and existing GitHub client methods as the starting point. Poll only active/linked PR context (or a currently open screen), and fetch check summaries for the relevant head SHA. Persist update time/cursor and \`next_due_at\`; do not repeatedly fetch full diffs, files, reviews or comment threads during background polling. Detailed PR data stays lazy-loaded when the user opens the corresponding view.

If a current schema/service does not provide enough stable metadata to schedule a PR/check cursor, add the smallest migration and test it. Do not use broad GitHub Search endpoints as a substitute for repository-scoped incremental sync.

### 6.6 Repository tracking

A repository becomes background-eligible only after the user explicitly discovers/selects/tracks it in ForgeDesk. Do not automatically sync every repository visible to the token. Preserve current repository/project association behavior.

## 7. Automatic Outbox dispatcher and reconciliation

### 7.1 Supported P0 operation scope

For this P0, automatically dispatch operations that the current application already enqueues and supports:

- \`update_issue\` for an already linked GitHub Issue: title, body/description, open/closed state and labels, subject to the existing \`field_ownership\` map.

Do not silently expand P0 to automatic creation of local-only Issues, comment synchronization, review submission, merge, close or other new write types. Those require their own operation contracts, user confirmation semantics, deduplication and reconciliation tests. Existing explicit PR actions remain explicit user actions.

### 7.2 Worker rules

- One Outbox worker consumes persisted eligible rows oldest-first per linked Issue/repository/account, within a bounded batch per tick.
- Do not dispatch rows for another workspace/account, an unlinked issue, an unresolved conflict, a disconnected account, a paused provider or a blocked budget bucket.
- Claim/lease a row atomically before network dispatch. A second wakeup/worker must not claim the same row.
- Expired \`dispatching\` lease after crash becomes \`outcome_unknown\`, never \`pending\` automatically.
- Keep request execution sequential by default (shared dispatcher max in-flight = 1); preserve at least one second between mutation requests.
- P0/P1 user operations outrank the background Outbox queue.
- Retry only known-safe/definite failures according to typed classification and configured limit/backoff.
- A dispatcher refusal before sending (user pause, budget gate, cancellation, no auth) must not be recorded as a remote mutation outcome. Leave the job pending/delayed or paused with a reason.
- Network timeout/reset after a mutation may have been sent => \`outcome_unknown\`; never blind-retry.
- Clear \`next_attempt_at\` only when a job becomes eligible; persist backoff and retry count.

### 7.3 Reconciliation

For \`outcome_unknown\`:

1. Transition to \`reconciling\` and issue a bounded read through the same dispatcher with class \`reconciliation\`.
2. Compare the exact fields present in the original payload with the current remote Issue state. Normalize label ordering and open/closed representation.
3. If all requested values match, mark \`succeeded\`, update remote link timestamps and stop.
4. If the remote state proves the update was not applied and retry is safe, transition to \`retry_safe\`; the worker may later return it to \`pending\`.
5. If the remote state differs ambiguously or a newer remote change conflicts with the local change, create/reuse \`sync_conflict\` rows and transition to \`needs_user_action\`.
6. If reconciliation read fails (offline, rate limited, auth expired), retain \`outcome_unknown/reconciling\`, persist next retry time and reason. Never treat failure to reconcile as proof that mutation failed.
7. If the user resolves the conflict, preserve existing local/remote field ownership semantics and enqueue a new idempotent update only for the fields that should be sent.

Reconciliation is operation-specific. Do not infer success from matching title alone if body/state/labels were also in the payload. Do not retry creation/comment operations in this P0 because those operation types are out of scope.

### 7.4 Crash and retry safety

- Simulate app death after each durable transition: before send, after send before response, after response before local commit, during reconcile, after conflict creation.
- Idempotency key/fingerprint must be deterministic and scoped to operation + remote entity + relevant payload/revision. Avoid unstable hashes where a stable cryptographic digest already exists.
- Do not put sensitive private payloads into API request log, scheduler payloads or diagnostic export.
- Keep the existing Outbox result summary or evolve it compatibly for monitoring; do not report skipped as success.

## 8. API Monitor and Sync settings UI

Implement truthful, localized operational visibility. Follow \`06_UI_UX_SPEC.md\` and reuse existing components/tokens. Do not invent a new dashboard layout.

API Monitor / Sync settings must show:

- dispatcher state (available/running/stopping/error);
- background worker state (disabled, idle, running, paused, waiting for budget, waiting for network, auth required, error);
- last successful sync and last attempt for each tracked resource/scope;
- next due time;
- current active job and queued counts by priority/class;
- Outbox counts by state (pending, dispatching, outcome unknown/reconciling, needs user action, failed definite);
- latest primary budget per API family/resource: observed remaining/limit/reset and observation age;
- background allowance/reserve as calculated values, with unknown/stale clearly shown as unavailable/zero;
- secondary pause status and retry-after/reset reason;
- last worker error/incomplete reason in safe, user-readable form;
- Pause/Resume control (already exists) and a manual “sync current context” action;
- manual full/backfill action only with estimated pages/cost, explicit warning and cancellation.

Requirements:

- English and Russian strings for every new state/action/error.
- Loading, empty, disconnected, offline, stale, paused, auth-required, permission error and partial/incomplete states.
- UI must not say “Active” or “Synced” when only a worker thread exists or a job was merely queued.
- Display UTC timestamps in the existing locale/time formatting utilities.
- Do not display full private issue body, raw response, token, Authorization header or unredacted transport details.
- Button actions must be real typed IPC and return success/failure from the host. No decorative controls.
- Keep all sync state local-first and render cached records without waiting for network.

## 9. Database migration and compatibility

Current baseline is schema v11. If no later schema exists on the implementation branch, use the next append-only migration (v12); if schema has advanced, use the next available version, never edit an applied migration.

Likely migration changes (add only what tests and source prove necessary):

- scheduler/read queue dedupe key, safe payload, due time, lease and completion/cancellation metadata on \`api_queue_job\`;
- Outbox retry/due/lease/failure classification fields on \`sync_outbox\`;
- cursor query fingerprint / any missing page-window metadata on \`sync_cursor\`;
- reliable account/principal identity linkage for budget observations and request log, including safe backfill/compatibility for existing nulls;
- indexes/constraints to avoid duplicate active read jobs and claim races.

Migration rules:

- All schema changes append-only; do not edit \`0001\`–\`0011\`.
- Keep SQLite STRICT/check constraints coherent with typed Rust state enums.
- Add upgrade tests from the current schema v11 and at least one older supported schema that contains real linked Issue/Outbox/cursor data.
- Confirm old rows remain queryable and maintainable; do not reset queues or wipe cursors during upgrade.
- Test foreign keys and indexes after migration.
- Backup/restore must preserve queue, Outbox, conflict and cursor state.
- No token/secret column or private payload logging may be added.

Do not add a migration if existing schema plus a small repository/service change is sufficient.

## 10. Error model and observability

Use structured error categories and stable UI-facing error codes. Never parse arbitrary user-visible text as the only source of retry classification.

Every queued job should expose enough sanitized diagnostic state to explain why it is waiting or stopped:

- \`rate_limit_primary\`, \`rate_limit_secondary\`;
- \`auth_required\` / \`permission_denied\`;
- \`network_unavailable\` / \`timeout\`;
- \`validation\` / \`provider_error\`;
- \`cancelled\`;
- \`conflict_needs_user_action\`;
- \`unknown_outcome\`.

Logs should include job ID, operation code, account pseudonymous ID, resource scope, attempt, state transition and elapsed time; never token, headers, raw request body, private Issue content or comments.

Expose sanitized diagnostics without sensitive payloads. Keep telemetry bounded and apply retention/cleanup appropriate to the existing request log.

## 11. Required tests and acceptance scenarios

Tests must use deterministic fake clocks, mock HTTP fixtures or local test servers. Do not require a live GitHub account for correctness tests. Never create/change/close real Issues or PRs in automated tests.

### Dispatcher and budget

1. REST and GraphQL requests submitted concurrently use one shared in-flight slot.
2. Priority ordering is respected; FIFO within the same priority.
3. Duplicate pending reads coalesce; mutations do not.
4. Mutation requests are at least one second apart across REST/GraphQL.
5. Pause blocks new REST and GraphQL data API requests; Resume wakes queued work.
6. Pause does not erase queue/cursor/Outbox state and does not corrupt an in-flight request.
7. Primary limit exhaustion pauses the right bucket until reset without probe loops.
8. Secondary limit from REST 403/429 or GraphQL 200+errors pauses the shared dispatcher for Retry-After / minimum backoff.
9. Permission 403 is not classified as a rate limit.
10. Unknown/stale budget yields zero background allowance.
11. Formula boundary cases: zero, negative/invalid, small limits, exhausted reserve, remaining equal to reserve, full remaining, and rounding.
12. Background usage is computed from ForgeDesk request telemetry since observation using actual cost where available, not \`limit - remaining\`.
13. Two different GitHub accounts never share observations/allowance.
14. Conditional REST 304 records \`not_modified\` without replacing cached body or charging a normal primary read.
15. GraphQL actual cost/remaining/reset are parsed and persisted; partial data/errors are classified correctly.

### Scheduler and lifecycle

16. Fresh install/start shows cached/local state and does not trigger a full import.
17. Worker starts once; repeated initialization does not create duplicate workers.
18. Disconnected/auth-expired state performs no repeated provider calls.
19. Pause, Resume, Disconnect, reconnect and Shutdown interrupt waits and update worker state.
20. Shutdown stops new dispatches, allows the in-flight operation to finish or records a recoverable state, joins the worker and preserves pending work.
21. Offline jobs back off with bounded jitter; network-restored event wakes scheduler.
22. User P0/P1 action is not starved by P3/P4 backlog.
23. Manual full sync can be cancelled; cancellation does not corrupt a committed page or cursor.
24. Only explicitly tracked repositories are scheduled.

### Cursor and incremental sync

25. First incremental run creates cursor state and applies pages idempotently.
26. Page data and page cursor commit atomically.
27. Simulated crash before apply causes the same page to be fetched/re-applied safely.
28. Simulated crash after apply does not cause missing or duplicate local entities.
29. Multi-page continuation resumes from persisted opaque page cursor after restart.
30. Query fingerprint change invalidates previous cursor and starts a new safe window.
31. Watermark overlap catches equal/late updated timestamps without duplicate local Issues.
32. \`last_success_at\` advances only after a complete window; partial/network failures retain previous success watermark and set incomplete reason.
33. Pull-request-shaped Issue records remain excluded from Issue import.
34. Remote deletion/absence does not hard-delete the local Issue.
35. PR/check polling refreshes active linked summaries only and does not bulk-fetch diffs/reviews/comments.
36. Full historical import remains manual and bounded.

### Outbox and reconciliation

37. Pending linked Issue update dispatches automatically after becoming eligible.
38. Existing local field-ownership rules determine which fields are sent.
39. Unresolved conflict prevents automatic overwrite.
40. Crash in \`dispatching\` becomes \`outcome_unknown\` and is reconciled before any retry.
41. Network timeout after possible send never blindly retries.
42. Pause/budget/auth rejection before network send leaves the mutation safely pending/delayed, not falsely unknown or succeeded.
43. Reconcile exact payload fields; only an exact match marks success.
44. Reconcile mismatch creates/reuses field conflict and moves to \`needs_user_action\`.
45. Reconcile read failure remains unknown/reconciling with delayed retry, not failed or success.
46. Definite validation/permission failures do not retry indefinitely.
47. Idempotency key prevents duplicate enqueue and job lease prevents parallel duplicate dispatch.
48. Resolving conflict local/remote produces the expected next state and Outbox behavior.
49. No automatic create-Issue/comment/merge/review operation appears unless separately implemented and specified.

### UI and persistence

50. API Monitor shows real worker/queue/Outbox/budget states and correct local timestamps.
51. All states and controls are localized in RU/EN.
52. Offline/stale/auth-required/paused/incomplete states are explicit; no false “synced” status.
53. Migration v11→new schema preserves all sync rows and backup/restore round-trips them.
54. Exported diagnostics contain no access/refresh token or raw private payload.
55. Existing API Monitor pause visual tests remain green in both languages.

## 12. Implementation stages and delivery boundaries

Work in coherent, testable slices. Every slice must compile and preserve main's existing behavior.

### Stage P0-A — runtime and durable read queue

- Add lifecycle-managed worker and wake/stop channel.
- Add persistent job claiming, dedupe, due time, lease and crash recovery.
- Connect worker state to \`AppState\`/Tauri shutdown without holding global state lock over network.
- Start idle; no bulk sync on startup.
- Unit/integration tests for lifecycle, claims, leases, dedupe and shutdown.
- Do not enable automatic repository polling yet.

### Stage P0-B — common REST/GraphQL dispatcher and budget enforcement

- Preserve PR #12 REST dispatcher and add GraphQL transport integration.
- Propagate correct request class/priority from caller.
- Implement principal-scoped bucket observations, cost accounting, stale observation gate, 10%/20% policy and shared secondary guard.
- Pause/Resume gates both data transports.
- Tests for concurrency/priority/budget/headers/GraphQL rate errors.

### Stage P0-C — incremental cursor-based sync

- Use tracked repository scopes, persisted cursor and query fingerprint.
- Add bounded per-page issue incremental apply/checkpoint with overlap watermark.
- Add active linked PR/check summary refresh only.
- Keep full historical import manual.
- Test restart, page boundary, partial failure and query change.

### Stage P0-D — automatic Outbox and reconciliation

- Run existing supported \`update_issue\` Outbox operations in the worker.
- Add leases/due retries and clear definite-vs-ambiguous classification.
- Preserve conflict semantics and exact-payload reconciliation.
- Test crash/timeout/pause/auth/rate limit/conflict paths.
- Do not add issue creation/comments/PR mutations here.

### Stage P0-E — API Monitor/Sync UX and end-to-end acceptance

- Show actual worker, queue, cursor, Outbox, budget and error states.
- Add typed IPC for manual refresh/cancel only where needed.
- Complete English/Russian text and visual gates.
- Run full CI and platform smoke-test checklist.
- Update PROJECT_STATE, audit, stage report, ADRs and Wiki pages with the verified commit and test evidence.

If a stage becomes too large, split it into separate PRs. Do not submit one enormous unreviewable patch.

## 13. Expected file-level impact

This is a search map, not permission to blindly rewrite every file. Reuse existing abstractions and touch only what the concrete implementation needs.

| Area | Existing likely files |
|---|---|
| REST dispatcher | \`crates/forgedesk-core/src/github_dispatcher.rs\`, \`github_client.rs\` |
| GraphQL transport | Add a focused module only if needed, e.g. \`crates/forgedesk-core/src/github_graphql.rs\`; register in \`lib.rs\` |
| Budget domain/repository | \`domain/api_budget.rs\`, \`db/repositories/api_budget.rs\` |
| Sync domain/repository | \`domain/sync.rs\`, \`db/repositories/sync.rs\` |
| Sync and Outbox algorithms | \`github_sync.rs\`, focused new worker/scheduler module |
| App service | \`app.rs\` |
| DB/migrations | \`db/migrations.rs\`, new numbered SQL migration, repositories |
| Tauri lifecycle/auth | \`src-tauri/src/state.rs\`, \`src-tauri/src/lib.rs\` |
| Tauri IPC | \`src-tauri/src/commands/mod.rs\`, \`src/lib/ipc.ts\`, \`src/types/ipc.ts\` |
| API Monitor/Sync UI | \`src/features/api-monitor/\`, existing \`src/features/settings/\`, Pinia stores |
| Localization/visual | \`src/i18n/en.ts\`, \`src/i18n/ru.ts\`, relevant component tests and \`scripts/visual-gate*.mjs\` |

Potential documentation changes during implementation:

- \`docs/PROJECT_STATE.md\`
- \`docs/PRODUCT_GAPS_AUDIT.md\`
- \`docs/stages/STAGE-4-report.md\`, \`STAGE-5-report.md\`, and a new stage report if appropriate
- \`docs/decisions/\` for any real architectural decision
- \`docs/wiki/\` synchronized pages

## 14. Non-goals for this P0

Unless the user explicitly expands the scope, do not implement:

- automatic creation of GitHub Issues from local-only tasks;
- GitHub comment creation/import/synchronization;
- automatic PR merge/close/review submission;
- full PR diff/files/review-thread polling in the background;
- real-time webhooks, relay server or any mandatory cloud backend;
- GitLab/Gitea live synchronization;
- full historical synchronization automatically;
- a new multi-user/account-sharing model;
- automatic changes to remote GitHub objects not already explicitly initiated through supported user actions;
- unrelated redesign/refactor of local Issues, Projects, PR screens or global UI.

## 15. Constraints and engineering quality

- Rust Stable, Tauri 2, Vue 3, TypeScript, SQLite. Windows and Linux first-class.
- No mandatory backend, Docker, Redis or external queue.
- Keep \`forgedesk-core\` independent of Tauri.
- SQLite WAL and serialized writes remain authoritative.
- Never run a long network call inside a transaction.
- Never log/store secrets. No raw token in queue, logs, error text, screenshots or reports.
- Do not weaken existing migration, authorization, permission, confirmation, field-ownership or backup protections.
- No unbounded loops, unbounded pagination, rapid polling, retry storms or silently dropped jobs.
- No fake statuses/controls. All controls must call a real typed command.
- No unrelated refactor or dependency addition without necessity and justification.
- No changes to UI geometry/theme/layout outside the exact requirements in \`06_UI_UX_SPEC.md\`.
- Keep API defaults centralized and documented.

## 16. Token efficiency — mandatory

\`04_TOKEN_EFFICIENCY_RULES.md\` is mandatory for every DeepSeek API call. In particular:

1. Perform one targeted repository reconnaissance for this task; write/update a compact project map/state note if needed. Do not repeatedly enumerate or reread the full repository.
2. Read \`docs/PROJECT_STATE.md\`, \`03_GITHUB_API_BUDGET_AND_SYNC.md\`, \`04_TOKEN_EFFICIENCY_RULES.md\`, relevant ADRs and only the source/test files for the current stage.
3. Keep the static warm prompt and document order unchanged across calls to maximize prompt-cache reuse. Send dynamic stage/task details after the stable prefix.
4. Keep \`docs/PROJECT_STATE.md\` current and compact (target 1,500–2,000 tokens); keep detailed historical evidence in stage reports.
5. Use minimal unified diffs for existing files. Print full source only for genuinely new files or when patch context is unsafe.
6. Do not resend full successful build/test logs. Filter failures to command, exit code, relevant error lines and short context.
7. Run targeted tests during iteration; run full checks once per stage/PR and whenever shared contracts change.
8. Do not rerun an unchanged successful test without a code/environment change.
9. Do not include large screenshots or complete source dumps in routine status.
10. The required quality/security tests in this specification are not optional and must not be removed for token savings.
11. Keep each model invocation focused on one coherent stage. Do not ask the user about routine implementation details already defined here.
12. End each call with the concise report format below.

Required per-call report:

~~~text
STATUS: DONE | PARTIAL | BLOCKED
SCOPE: <one-line stage objective>
CHANGED:
- <path> — <reason>
TESTS:
- <command> — PASS/FAIL/SKIPPED (reason)
RISKS:
- <only real remaining risks>
NEXT:
- <one concrete next task>
~~~

Track usage metrics locally where available: prompt/input, output, cache hits/misses, model, stage and duration. Do not send source code or secrets to third-party usage analytics.

## 17. Git workflow and delivery requirements

- Start from current \`main\`, not a stale local branch. Verify \`main\` is at/descended from \`e380c58098878080b7d746f3cad7889effdc2fb6\`.
- Use a dedicated feature branch, e.g. \`feat/github-sync-runtime-p0\`.
- Keep PRs focused by stage. A PR must state exactly which P0 stages it completes and what remains.
- Never merge a PR, force-push, delete branches or perform destructive remote operations unless the user explicitly authorizes that action.
- No tests may perform live destructive GitHub writes.
- CI must run against the exact PR head. Do not report a previous green run for a newer commit.
- Preserve a clean reviewable diff and do not include generated build artifacts, local DBs, logs, credentials or screenshots beyond the established visual-gate artifacts.
- Before declaring DONE: fetch/check PR head, CI jobs, changed files, schema migration tests and source diff; verify no accidental scope expansion.

### Required checks

At minimum, run the relevant targeted tests for each stage, then the complete repository gate before declaring the overall P0 done:

~~~bash
cargo fmt --all --check
cargo clippy -p forgedesk-core --all-targets -- -D warnings
cargo test -p forgedesk-core
npm run typecheck
npm test
npm run build
node scripts/visual-gate0.mjs
~~~

Also run project scripts/CI jobs required by the current workflow, optional AI checks if shared workspace dependencies/build graph were touched, and Windows/Linux Tauri compile jobs through GitHub Actions. Do not claim native interactive runtime verification based on a compile-only job.

### Manual smoke checklist (must be reported separately from CI)

- Windows: connect GitHub, import/track a test repository, confirm incremental sync, edit a linked Issue offline/online, see Outbox dispatch and reconcile, pause/resume, close/reopen app and verify queue/cursor recovery.
- Linux: repeat the same flow with OS keyring available; also test session-only fallback where keyring is unavailable.
- Test with an isolated disposable repository and harmless Issue fields. Do not merge/close/delete the test repository or its Issues as part of smoke testing.
- Verify offline, token expired, permission revoked, rate-limit, pause/resume, shutdown mid-request and restored network scenarios.
- Report native smoke tests as NOT RUN until they have actually been performed.

## 18. Starter prompt for DeepSeek API

Use the project warm prompt from \`05_DEEPSEEK_PROMPTS.md\` unchanged. Keep all six project documents in the established order. Then submit this task prompt:

~~~text
Continue ForgeDesk from the current main branch. Implement the remaining P0 GitHub sync runtime according to docs/implementation/P0_REMAINDER_DEEPSEEK_TZ.md.

Mandatory context:
- Read docs/PROJECT_STATE.md.
- Read the relevant sections of 03_GITHUB_API_BUDGET_AND_SYNC.md.
- Follow 04_TOKEN_EFFICIENCY_RULES.md for every call.
- Preserve the PR #12 REST dispatcher and pause gate already merged at e380c58098878080b7d746f3cad7889effdc2fb6.
- Read docs/decisions/ADR-0006-sync-safety.md and only the source/tests relevant to the current stage.
- Do not reread the entire repository after targeted reconnaissance.

This is an execution assignment. Do not return only a plan or pseudocode. Implement one coherent stage at a time, add the required tests, run them, update docs/PROJECT_STATE.md and the corresponding stage report, and open a focused PR when the stage is reviewable.

Start with a compact source audit to confirm the exact current schema, dispatcher, scheduler, cursor, Outbox and UI state. Do not ask routine questions; choose the simplest safe implementation consistent with the specification and record genuine architectural decisions in an ADR.

Non-negotiable:
- No network requests inside SQLite transactions.
- Do not hold AppState's global mutex across network I/O.
- One shared REST+GraphQL dispatcher, one in-flight slot per principal, shared secondary guard and priority scheduling.
- 10% background cap plus 20% reserve; stale/unknown budget means zero background allowance.
- sync_outbox is canonical for writes; api_queue_job is canonical for scheduled/read work; do not create duplicate queue systems.
- Persist page cursor and watermark separately; commit page data and cursor atomically; resume after restart.
- Outbox mutations are never blindly retried after ambiguous outcomes; reconcile exact payload fields first.
- P0 automatically dispatches only already-supported update_issue Outbox records. Do not add Issue creation/comments/PR mutations in this task.
- Only explicitly tracked repositories are background-eligible; full historical import remains manual.
- Preserve the UI/UX specification, API Monitor Pause/Resume, Windows/Linux support, migrations and security.
- Never claim CI/native smoke checks that were not actually run.
- Never merge any PR without explicit user authorization.

Work in P0-A → P0-B → P0-C → P0-D → P0-E order from the spec, but split into focused PRs where that improves reviewability. Each PR must be green at its exact head before it is reported ready.

Return only the concise STATUS/SCOPE/CHANGED/TESTS/RISKS/NEXT report unless a real blocker needs explanation.
~~~

## 19. P0 completion report template

When all P0 stages are complete, update this section in a stage report with:

- exact main/PR merge SHA;
- per-stage PRs and changed-file summary;
- schema version and migration upgrade test results;
- exact GitHub Actions run URL and all job conclusions for the exact tested SHA;
- targeted and full test commands/results;
- Windows and Linux native smoke-test status (separate from compile CI);
- known limitations and P1 follow-ups;
- confirmation that no test created or modified a real GitHub resource unexpectedly;
- updated PROJECT_STATE and Wiki links.

Do not mark P0 complete while any P0 acceptance criterion above is unverified.
