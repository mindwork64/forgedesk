# 07. P0 — Background Sync, API Budget Enforcement, Outbox Worker and Incremental Cursors

**Project:** ForgeDesk  
**Status:** Implementation specification for DeepSeek API  
**Baseline:** main at e380c58098878080b7d746f3cad7889effdc2fb6 (2026-11-18)  
**Previous slice:** PR #12 merged. Shared GitHub REST dispatcher, telemetry and API Monitor pause gate are implemented.  
**Goal:** finish the remaining P0 synchronization runtime without rebuilding existing foundations or adding a backend service.

---

## 1. Mission and definition of done

Implement a production-usable, persistent, quota-aware GitHub synchronization runtime for the existing local-first desktop application.

P0 is complete only when all of the following are wired into the real application lifecycle and covered by deterministic tests:

1. One background scheduler/worker starts when the app is running and GitHub is connected. The UI must show the SQLite cache immediately and must not wait for network work.
2. Only relevant, already tracked GitHub resources are scheduled. Startup must not crawl every repository or historical Issue.
3. All GitHub REST work uses the existing shared dispatcher. Background work obeys the 10% background allowance and 20% safety reserve, and yields to interactive work.
4. Existing durable Outbox rows are processed automatically. Ambiguous mutation outcomes are reconciled before any retry.
5. Existing sync cursors are used by production sync code. Page progress and watermarks survive app restart and are advanced transactionally.
6. Sync settings and API Monitor show truthful worker, queue, retry, pause, quota and freshness state.
7. Windows and Linux remain supported. Offline local work remains fully functional. No required backend or AI dependency is introduced.

Creating tables or types that are never used by production code does not count as implementation.

## 2. Verified baseline — preserve and extend

The repository was inspected after PR #12 merged. Current baseline commit is e380c58098878080b7d746f3cad7889effdc2fb6.

### 2.1 Already implemented

- Rust Stable + Tauri 2 + Vue 3/TypeScript + SQLite. Local-first, single-user desktop application.
- Schema v11. Migrations are append-only in crates/forgedesk-core/src/db/migrations.rs.
- GitHub Device Flow authentication, refresh and OS keyring handling in crates/forgedesk-core/src/github_auth.rs.
- GitHub REST client in crates/forgedesk-core/src/github_client.rs.
- PR #12 added crates/forgedesk-core/src/github_dispatcher.rs. Authenticated GitHubClient REST requests use a process-wide single-flight gate. Request telemetry and REST rate-limit observations are persisted. Fingerprints do not include access tokens.
- PR #12 added a working API Monitor Pause/Resume control. api_control.paused is checked before authenticated REST requests made through GitHubClient.
- OAuth Device Flow, token exchange and refresh remain outside the REST dispatcher and are intentionally not blocked by the REST pause.
- Existing tables: api_budget_observation, api_request_log, api_queue_job, api_control (migration 0004).
- Existing tables: remote_issue_link, sync_cursor, sync_outbox, sync_conflict (migration 0005).
- Outbox idempotent enqueue and closed state machine exist. Local updates to linked GitHub Issues can enqueue update_issue payloads.
- Manual bidirectional sync of already linked GitHub Issues exists for title, body, open/closed status and labels. Conflict resolution is available in Settings → Sync.
- Manual Issue import and bounded REST pagination exist in crates/forgedesk-core/src/github_sync.rs.
- AppService::dispatch_github_issue_outbox manually processes update_issue rows and attempts GET reconciliation for outcome_unknown rows.

### 2.2 Current limitations confirmed in code

- No production background scheduler or worker lifecycle exists.
- api_queue_job is displayed by API Monitor but is not a functioning durable execution queue.
- RequestClass includes BackgroundSync, ManualFullSync and Reconciliation, but production GitHubClient creation defaults to InteractiveRead. execute currently selects InteractiveWrite for non-GET calls. There is no production way to retain a background class for a mutation.
- The dispatcher gate serializes requests with a synchronous mutex; it does not provide priority-aware admission between waiting calls.
- The dispatcher checks manual pause and known rate observations, but does not yet enforce the full per-window 10% allowance / 20% reserve rule or a stale-observation rule.
- Dispatcher observations/log rows currently use provider_account_id = NULL. They are not fully scoped to an authenticated GitHub principal.
- sync_cursor functions exist, but production sync does not persist a complete page continuation/watermark workflow. Manual import still uses start_page.
- Outbox is manually dispatched with owner/repository filters. It is not scheduled automatically.
- The manual Outbox path currently treats every mutation error as outcome_unknown. The worker must classify confirmed HTTP rejection separately from ambiguous network/server outcomes.
- Current Outbox dispatch only supports update_issue. Do not assume comments, issue creation or other mutations are supported.
- AppState::with_service currently holds the outer app-state mutex while the service closure runs. A worker must not hold this mutex over network I/O.
- API Monitor currently labels background sync as not implemented. Change that label only after a real worker is running and observable.

### 2.3 Required source documents

Read the latest docs/PROJECT_STATE.md first, then only relevant parts of these documents:

- 01_PRODUCT_PROJECT.md — product and offline/local-first contract.
- 02_TECHNICAL_SPEC.md — architecture boundaries and testing conventions.
- 03_GITHUB_API_BUDGET_AND_SYNC.md — sections 5, 8, 9, 10, 12 and 13.
- 04_TOKEN_EFFICIENCY_RULES.md — mandatory DeepSeek token/context/cache rules.
- 05_DEEPSEEK_PROMPTS.md — stable warm prompt and stage prompt convention.
- 06_UI_UX_SPEC.md — Sync settings/API Monitor design and UI Gates.
- docs/decisions/ADR-0006-sync-safety.md — accepted Outbox/cursor/conflict decisions.

Do not repeatedly read or send all documents. This specification refines the remaining P0 work. Where an older historical snapshot conflicts with Section 2 or the newest PROJECT_STATE addendum, the newest verified state wins.

## 3. Scope boundaries

### 3.1 Included

- One application-owned background runtime.
- Priority-aware use of the existing GitHub REST dispatcher.
- Adaptive REST background budget allocation and reserve enforcement, per authenticated provider account and resource bucket.
- Persistent queue scheduling, deduplication, leases, restart recovery, cancellation and retries.
- Automatic Outbox processing/reconciliation for the currently supported update_issue operation.
- Incremental sync for existing GitHub Issue support and supported tracked PR/check summaries.
- Real worker/queue/cursor status in existing Sync settings and API Monitor screens.
- RU/EN translations, tests, diagnostics and documentation.

### 3.2 Explicitly out of scope

Do not silently expand P0 into the following:

- GitLab/Gitea live synchronization.
- New comment synchronization or automatic comment creation.
- Automatic publishing of all local-only Issues as new GitHub Issues. Existing remote mutation scope remains update_issue only.
- Full GraphQL client rollout. GraphQL model fields can remain ready, but do not claim GraphQL worker support without an actual transport through the same dispatcher.
- GitHub Projects V2 implementation, webhook relay/server, cloud sync, remote Git push/pull/rebase.
- Full historical sync on startup. Historical/bulk sync remains a deliberate, cancellable user action.
- Broad UI redesign. Follow 06_UI_UX_SPEC.md and reuse existing components/tokens.
- AI/DeepSeek runtime dependencies in ForgeDesk.

If a requirement appears to need an excluded capability, report the boundary/blocker. Do not imitate a missing feature or label a partial feature complete.

## 4. Architecture and ownership rules

### 4.1 One source of truth per concern

| Concern | Canonical owner | Rule |
|---|---|---|
| Local business data | SQLite domain tables | SQLite remains authoritative for ForgeDesk-local Issues/Projects. |
| Remote identity mapping | remote_issue_link and pull-link tables | Use stable provider IDs and explicit field ownership. |
| Mutation intent/state | sync_outbox | Canonical mutation record; never replace it with an in-memory queue. |
| Incremental progress | sync_cursor | One cursor per provider account/resource/canonical scope/query shape. |
| Scheduled execution | api_queue_job | Execution envelope referencing Outbox or cursor; not a second mutation source of truth. |
| Telemetry/budget | api_request_log and api_budget_observation | No tokens, Authorization headers or full private payloads. |
| Global REST pause | api_control | Existing Pause/Resume remains authoritative. |
| Process lifecycle | one Tauri-owned runtime | No detached threads or per-command worker creation. |

For queue rows, use a source_kind/source_id relationship: outbox rows reference sync_outbox.id; cursor rows reference sync_cursor.id. The queue must not duplicate mutation payloads. A deterministic unique job_key must prevent duplicate work from repeated scheduler ticks.

At most one active job per canonical source/resource by default. Do not add parallel page jobs unless a correctness argument and tests demonstrate safety.

### 4.2 Dispatcher context

Extend github_dispatcher.rs. Do not create a second HTTP client, semaphore or request path.

Every call must carry typed context:
- request class: interactive read/write, foreground refresh, background sync, manual full sync or reconciliation;
- priority consistent with 03_GITHUB_API_BUDGET_AND_SYNC.md;
- provider-account identity;
- API family/resource;
- operation code and safe fingerprint;
- retry classification and optional cancellation/deadline.

Add a typed constructor or scoped context for GitHubClient (for example, new_with_context). Do not infer class from HTTP verb alone. A PATCH sent by background worker must remain BackgroundSync for accounting; mutation pacing is a separate concern.

Interactive UI calls use interactive context. Outbox worker uses BackgroundSync for dispatch and Reconciliation for verification GETs. Explicit full sync uses ManualFullSync. All pass through the same dispatcher.

Keep max_in_flight_per_principal = 1. Keep at least one second between mutative REST requests across callers. Reads do not need an unconditional one-second delay unless rate-limit signals require it.

The current mutex has no priority semantics. Add priority-aware admission using a bounded synchronous queue/condition variable or an already-established suitable mechanism. Do not rely on std::sync::Mutex fairness. Never hold the admission lock while performing HTTP or sleeping for mutation pacing. Interactive work must be selected before waiting background work.

### 4.3 Provider-account scope

Budget and queue data must belong to the authenticated GitHub principal.

- Reuse/upsert the existing provider_account record for provider=github, host=github.com and connected login.
- Pass a stable local provider_account.id into dispatcher context and persist it in observations, request logs and queue jobs.
- Never persist or log raw access tokens. Do not treat a token hash as an account identity.
- On disconnect/revocation, pause provider jobs but retain Outbox/cursor progress.
- Never resume account A's pending mutations under account B credentials.

### 4.4 Threading and lock boundaries

- Create one worker during Tauri setup and stop it on app exit.
- Core sync code must not depend on Tauri UI APIs.
- Do not hold AppState's outer mutex during HTTP. Clone Arc handles under lock and release it before service/auth operations, or use an equivalent safe ownership model.
- AppService/SQLite remains serialized by the existing Database mutex. Never hold a SQLite transaction/connection lock across HTTP.
- Use short transactions: read/claim → commit → network → apply result in a new transaction.
- Worker and UI must share the same logical service/database and authenticated account. Do not open extra AppService instances that trigger unrelated startup backups/work.
- Lifecycle must be explicit: start once, wake, stop scheduling, finish/abort safe reads within bounded HTTP timeout, persist state, join thread. No detached thread.
- On restart, expired running read jobs may return to queued. An Outbox mutation left in dispatching must become outcome_unknown and be reconciled before retry.

### 4.5 Wake-up conditions

Wake the scheduler for:
- next due sync resource;
- eligible Outbox retry/reconciliation;
- queue retry time;
- rate reset / Retry-After / secondary pause expiry;
- explicit Sync now;
- foreground/open-entity stale refresh;
- shutdown.

Use a condition variable/channel, not a tight polling loop. Centralise intervals and jitter.

## 5. Scheduler policy

### 5.1 Eligible resources

Schedule only what the user already connected, selected, linked or imported:
- GitHub repositories represented in local repository registry and linked/selected for a ForgeDesk Project;
- Issues already imported/linked or included by an explicit repository import/sync selection;
- supported PR/check summaries related to tracked repositories or linked Issues/PRs.

Do not enumerate every accessible repository or crawl account history at startup. Repository discovery is not implicit full sync.

### 5.2 Lifecycle

- Startup: render local cache immediately. Worker starts idle. If no GitHub account, make zero GitHub REST calls. If connected, enqueue only minimal due work.
- Auth completed/refreshed: validate principal and wake scheduler. Create/update provider-account mapping and enqueue only due tracked resources.
- Foreground/open entity: prioritise that context if stale; no account-wide refresh.
- Manual refresh: enqueue at interactive/foreground priority.
- Network/API failure: classify and back off. No retry loops.
- API Monitor Pause: do not start background work. Dispatcher is still the final gate for all REST calls. Persist jobs while paused.
- Disconnect/401: stop provider work, preserve jobs/cursors and surface reconnect requirement.
- Exit: graceful stop and restart recovery as above.

### 5.3 Initial intervals

These are centralised defaults, not promises that every resource will be polled on schedule:

| Resource | Initial target | Eligibility |
|---|---:|---|
| Active tracked PR/check summary | 10–15 min | Tracked/open/active or visible PR context, budget permitting. |
| Active tracked Issues/repository Issue metadata | 30–60 min | Explicitly linked/imported/selected repositories only. |
| Inactive tracked repository metadata | 4–8 hours | Low priority. |
| Old/closed Issue history, comments, reviews, diffs/files | On demand | No routine polling in this P0. |
| Full repository/history sync | Manual | Must be cancellable and show estimated pages/cost. |

Apply bounded jitter (initially ±10%). If budget is insufficient, defer work; do not burst later. No sync daemon after the desktop application has closed.

### 5.4 Priority

Use the established order:
1. P0_INTERACTIVE_WRITE
2. P1_INTERACTIVE_READ
3. P2_ACTIVE_CONTEXT
4. P3_BACKGROUND_SYNC
5. P4_MANUAL_BULK_SYNC

Bulk sync is user initiated but cannot starve ordinary UI work. Yield the dispatcher between pages.

## 6. Adaptive API budget and rate limits

### 6.1 Required formula

For each principal + API family + resource + reset window:

background_allowance_total = min(floor(0.10 * observed_limit), max(0, observed_remaining_at_allocation - safety_reserve))

safety_reserve = ceil(0.20 * observed_limit)

Unknown or stale observations mean zero background allowance until a fresh trustworthy observation exists. Interactive user actions may use budget outside background allowance, but cannot bypass real GitHub limits or global pause.

The reserve is a safety boundary, not a guarantee. Other applications using the account can consume quota between observations.

### 6.2 Do not re-grant 10% after every request

Persist a window allocation keyed by provider account, API family, resource and reset_at (or an equivalent stable reset-window key). Store:
- reset-window identity;
- limit and observation used for allocation;
- total allowed background cost;
- allocation timestamp;
- cumulative background cost consumed in the window;
- latest provider remaining observation and timestamp.

A newer observation in the same window can reduce available background work. It must not increase the window's original allowance cap. A new reset window gets a fresh allocation only after a valid observation.

A dedicated allocation table is acceptable if required. Add it in a new migration. If existing tables can calculate the invariant safely without double-counting, document and test that implementation before choosing it.

### 6.3 Cost accounting

- Current REST calls use conservative cost 1 unless endpoint-specific evidence requires more.
- Account by request class and actual API resource. Count confirmed limited requests conservatively; do not assume every transport failure consumed quota.
- Reconcile estimate against actual x-ratelimit headers after each response.
- Keep REST resource buckets separate (core, search and any resource returned by GitHub).
- Do not claim GraphQL background support until a GraphQL client is implemented through the same dispatcher.

### 6.4 Freshness and rate-limit handling

- Centralise maximum age for background-eligible observations; initial default 15 minutes. Unknown/older observations block background work until refreshed safely.
- x-ratelimit-remaining=0 blocks the affected primary bucket until x-ratelimit-reset. Do not probe before reset.
- Secondary limit: respect Retry-After. If absent, wait at least 60 seconds unless primary reset is longer. Repeated secondary errors use exponential backoff with jitter; cap automatic attempts at 3.
- Distinguish 403 permission errors from primary/secondary rate limits. Never classify all 403 as rate limits.
- Secondary pause is global to the GitHub principal, not just one endpoint.
- Expiry/success may clear secondary pause but must never clear api_control.paused.
- OAuth Device Flow/refresh remain outside REST pause by the design of PR #12.

### 6.5 Background admission conditions

Before background dispatch, check:
- manual API pause;
- secondary pause;
- primary bucket exhaustion;
- observation freshness;
- remaining headroom after reserve;
- remaining reset-window background allowance.

Background work yields to interactive requests. Manual full sync does not bypass reserve. Interactive user actions remain subject to actual provider limits.

## 7. Persistent queue and migration

Current baseline schema is v11. Re-check CURRENT_SCHEMA_VERSION before coding. Add an append-only migration v12 or next available version; never edit migrations 0001–0011.

Extend api_queue_job only as required. Likely additions:
- unique deterministic job_key;
- source_kind and source_id;
- safe payload_json for read job parameters only;
- next_attempt_at;
- started_at and lease_expires_at;
- optional cancel_requested;
- indexes for due jobs, state, source uniqueness and lease recovery.

Associate queue rows with provider_account_id. Backfill legacy rows safely or leave nullable with an explicit legacy policy.

Do not add an unrelated generic job framework. Queue is the execution envelope. Outbox remains the mutation source of truth. Cursor remains the read-progress source of truth.

Atomicity:
1. Enqueue/deduplicate in a transaction.
2. Atomically claim one due job and set running + bounded lease; commit.
3. Perform HTTP without a DB transaction.
4. Apply response/cursor/outbox transitions in a new short transaction.
5. Recover expired leases on startup/tick.
6. A duplicate tick must not create duplicate jobs or mutation requests.

## 8. Automatic Outbox worker

### 8.1 Supported operation

Automatically process existing update_issue Outbox jobs for already linked GitHub Issues only. The manual github_dispatch_issue_outbox command remains available but delegates to the same service implementation. No second dispatch implementation.

Do not add comments, Issue creation, PR merge or other new remote mutations in this P0.

### 8.2 State machine

Preserve ADR-0006:

pending → dispatching → succeeded | failed_definite | outcome_unknown → reconciling → succeeded | retry_safe | needs_user_action

Do not add outcome_unknown → pending or outcome_unknown → succeeded direct transitions. Preserve existing cancel/terminal states.

### 8.3 Dispatch procedure

For each eligible Outbox row:
1. Verify authenticated provider account matches the row.
2. Resolve remote_issue_link, repository full name and remote Issue number.
3. Skip safely if unlinked, absent or belongs to another account.
4. Stop if unresolved local conflicts exist.
5. Check pause, rate budget, retry time and priority.
6. Transition pending → dispatching and increment attempts transactionally.
7. Send through shared dispatcher with BackgroundSync (or foreground context for manual dispatch).
8. On confirmed success, persist remote timestamps/link data and mark succeeded.
9. On confirmed rejection, mark failed_definite. Set retry_safe only if the response proves the mutation was not applied and policy allows retry.
10. On ambiguous transport/server result where the request may have been applied, mark outcome_unknown.
11. Reconcile with a separate low-priority GET through RequestClass::Reconciliation.

### 8.4 Recovery and reconciliation

- A row left dispatching after process restart becomes outcome_unknown with an explanatory reason.
- outcome_unknown must become reconciling before remote verification.
- Compare only fields present in the Outbox payload. Normalise values where necessary.
- If remote state matches, update link timestamps and mark succeeded without another mutation.
- If remote state proves the mutation was not applied, transition retry_safe and return to pending only when due and within budget.
- If state differs or cannot be established, create/preserve conflict and mark needs_user_action. A failed GET is not proof that the mutation failed.
- Reconciliation is bounded, backoff-controlled, visible and lower priority than user work.
- Classify confirmed HTTP responses separately from transport ambiguity. Do not create infinite outcome_unknown loops for auth/permission/validation errors.

### 8.5 Retry defaults

- Safe reads: bounded exponential backoff with jitter, max 3 automatic attempts per classified failure.
- Confirmed rate-limit rejection: wait Retry-After/reset; retry only when known safe.
- 401: stop provider queue and require reconnect.
- 403 permission: visible permission error/needs_user_action unless explicit rate-limit signal.
- 404: distinguish missing from private-resource access denial. No infinite retries.
- 422: failed_definite/needs_user_action until input changes.
- Network timeout after mutation dispatch or uncertain 5xx: reconcile first.
- Max 3 automatic mutation attempts per Outbox operation. User may explicitly retry after reviewing terminal state.

## 9. Incremental cursors and watermarks

### 9.1 Cursor identity

Use existing unique key provider + account_id + resource + scope:
- provider=github;
- account_id is the local stable identity of the connected GitHub user, never a raw token;
- resource is a stable logical type (issues, pull_summaries, check_summaries);
- scope is canonical repository or entity scope.

Persist a query-shape/filter/sort fingerprint either in a new column or a canonical scope key. If filter, sort, page size, API version or selected field set changes incompatibly, invalidate the old cursor. Never reuse a cursor across accounts/repositories.

### 9.2 Page cursor vs watermark

- page_cursor stores opaque continuation for an in-progress run. Prefer actual Link next URL/cursor from provider response; do not invent it.
- watermark stores last fully committed update boundary. It is separate from page_cursor.
- Use overlap window initially 2–5 minutes when starting the next incremental run. Duplicate refetch is expected; upsert/dedup makes it safe.
- Use stable ordering and updated_at/since semantics where supported. Do not rely solely on page number across a changing dataset.
- Do not use Search API for full synchronization.

### 9.3 Transactional page procedure

1. Read current cursor/query identity.
2. Fetch one page through GitHubClient/shared dispatcher with no DB transaction open.
3. Validate response and continuation.
4. In one SQLite transaction: upsert all remote entities/links, create/update conflicts, and advance page_cursor only after every record on the page is applied.
5. On final page, atomically clear page_cursor, advance watermark to max fully processed updated_at, set last_success_at and next_due_at, and clear transient incomplete/error state.
6. If transaction fails/crash occurs before commit, re-fetch/re-apply the page safely. Upsert and remote-ID uniqueness must make replay idempotent.
7. On failed/partial page, preserve page_cursor/watermark, record last_attempt_at, error and incomplete_reason.
8. Do not update last_success_at for a partial multi-page run.

### 9.4 Deleted/inaccessible objects

- Absence from a page is not proof of deletion.
- Use tombstone only with reliable provider evidence.
- A 404 can mean missing permission/private resource. Do not unlink or delete local data silently.
- Local Issue/Project survives inaccessible GitHub links.

### 9.5 Manual full sync

- Explicitly user-triggered and cancellable.
- Estimate pages and REST cost from last-known data; clearly label estimate.
- Bounded pages, yield dispatcher between pages.
- Persist page cursor after each committed page.
- No automatic history/comments/reviews/diffs in this P0.

## 10. UI and user-facing behavior

Follow 06_UI_UX_SPEC.md exactly. Reuse existing layout, components, tokens and patterns.

### 10.1 Sync settings

Expose only functional controls/status:
- background sync enabled/disabled;
- connected GitHub account/auth status;
- last successful sync and next due;
- Sync now for selected/current scope;
- cancellable manual full sync only if estimate is available;
- pending/failed/unknown/reconciliation-required Outbox counts;
- unresolved conflict count/link;
- clear reason for pause/defer: user pause, quota, stale observation, auth, network or conflict.

Do not show Background sync: Active unless the worker is actually running and observable. Distinguish Disabled, Waiting for connection, Paused by user, Rate limited, Running, Idle, Auth required and Error.

### 10.2 API Monitor

Retain PR #12 Pause/Resume. Add:
- worker state and last heartbeat;
- active job/resource (sanitised);
- queued/running/paused/failed/outcome_unknown/done counts;
- next due and last success;
- background allowance total/remaining, reserve and reset per REST resource;
- latest slowdown/secondary pause reason;
- Outbox pending/reconciliation backlog.

Distinguish GitHub-reported remaining (last observation), ForgeDesk observed spend and estimates. Avoid false precision.

Background-sync enable/disable controls scheduling only; they do not toggle global REST pause. Global API pause blocks all authenticated REST classes. Stopping background sync must not block interactive UI reads/writes.

### 10.3 UX quality

- English and Russian strings in current i18n files.
- Existing toast/error-state conventions.
- Keyboard/focus/loading/disabled states follow current components.
- No fake buttons, sample jobs, placeholder progress or hardcoded Active status.
- Update scripts/visual-gate0.mjs and relevant UI Gates to assert real state in RU/EN.

## 11. Implementation stages and deliverables

Use vertical reviewable stages. Keep main unchanged. Create feature branch(es) and PR(s) to main. Do not merge. Every stage ends buildable, with docs/PROJECT_STATE.md and a stage report updated.

### Stage A — worker lifecycle and dispatcher context

- One Tauri-owned runtime/supervisor with start/stop/wake.
- Fix AppState lock boundaries; UI and worker must not hold outer mutex during HTTP.
- Typed request context and priority-aware dispatcher admission.
- Correct request classes for interactive, foreground, background, manual full sync and reconciliation.
- Prove startup/offline/no-auth/shutdown/restart behavior before periodic sync is enabled.

Acceptance: one worker only; no REST request without auth; one in-flight per principal; interactive requests win over waiting background jobs; worker stops cleanly; no DB transaction or outer AppState lock spans HTTP.

### Stage B — durable queue and adaptive budget

- Add next append-only migration for queue leases/dedup/retry and budget-window allocations if needed.
- Atomic claim/recovery, job keys, bounded leases and backoff.
- Account-scope queue/observation/log data.
- Implement 10% per reset-window background allowance, 20% reserve, stale-observation zero allowance, primary/secondary guard.
- Keep manual API pause separate and final.

Acceptance: repeated ticks cannot duplicate jobs or regrant the same 10% window; lower external remaining reduces background availability; unknown/stale observation gives zero background work; background reserve alone does not block user-interactive work; actual primary/secondary and user pause are enforced.

### Stage C — automatic Outbox worker

- Reuse/refactor one Outbox service path for both manual command and worker.
- Auto-process update_issue only, for connected account and existing remote link.
- Add safe error classification, retry schedule, interrupted-dispatch recovery and low-priority reconciliation.
- Preserve conflict and user retry behavior.

Acceptance: offline edit remains local and queued; reconnect dispatches automatically; ambiguous mutation is never blindly replayed; restart during dispatch triggers reconciliation; conflicts stop overwrites; manual trigger still works through same code.

### Stage D — persistent incremental sync

- Wire cursor storage into production Issue sync and currently supported tracked PR/check summary refresh.
- Fetch one page at a time, apply transactionally, persist continuation/watermark/next due.
- Add overlap window and query-shape invalidation.
- Resume partial work after restart without starting over or skipping changes.

Acceptance: cursor survives restart; entity upsert and cursor advancement are atomic; interrupted page is safely replayable; only full completion advances watermark/last_success_at; filter/account/repository change invalidates cursor.

### Stage E — UI, diagnostics and end-to-end gates

- Wire Sync settings/API Monitor to real runtime/queue/cursor data.
- Update RU/EN, deterministic visual gates and tests.
- Run Windows/Linux CI-equivalent checks and document native smoke tests.

Acceptance: no false Active status; pause/queue/next-due/reconciliation display is accurate; UI Gates and CI pass; app remains responsive/offline while worker is idle/backing off.

## 12. Mandatory test matrix

Use deterministic clocks, seeded SQLite and local mock HTTP server. Never rely on live GitHub quota, destructive production writes or real-time sleeps. Inject clock/sleeper or test pure scheduler functions.

### Scheduler/lifecycle
1. No account: idle worker, zero REST calls.
2. No tracked resource: no repository-wide scan.
3. Local bootstrap is immediate and does not wait for network.
4. Repeated start: one worker.
5. Foreground refresh wins over queued background page.
6. Jitter and interval bounds work; no busy loop.
7. Network failure backs off; recovery wakes due work.
8. Exit stops new work; restart recovers due queue/cursors.
9. No AppState mutex/DB transaction held over HTTP.
10. Worker and UI share one in-flight gate.

### Dispatcher/budget
11. GET/POST/PATCH retain intended request class; background mutation is not reclassified interactive.
12. No separate worker HTTP client.
13. Mutation spacing >= 1 second across callers.
14. Allowance formula for high/low/zero/unknown budgets.
15. Total background use never exceeds 10% window allocation.
16. Reserve remains protected.
17. New lower remaining observation reduces background availability.
18. Stale/unknown observation blocks background work.
19. New reset window receives new allowance only after valid observation.
20. Manual pause persists and blocks every authenticated REST class; OAuth Device Flow remains outside.
21. Primary remaining=0 waits for reset; no probe.
22. Secondary limit honors Retry-After, or >=60 sec plus bounded backoff.
23. Permission 403 is not treated as rate limit without explicit signal.
24. Account A queue/budget never used by account B.

### Outbox
25. Duplicate idempotency key returns existing row.
26. Pending update is automatically dispatched when eligible.
27. Confirmed success marks succeeded and updates remote link timestamp.
28. 401 stops provider queue and preserves Outbox.
29. Permission/validation errors do not retry forever.
30. Timeout after send becomes outcome_unknown.
31. Simulated restart during dispatch recovers to outcome_unknown.
32. outcome_unknown always enters reconciling first.
33. Matching remote state marks success with no duplicate mutation.
34. Proven-not-applied can become retry_safe and only retry after backoff/budget.
35. Divergent/ambiguous state becomes conflict/needs_user_action.
36. Reconciliation GET uses reconciliation class and yields to interactive.
37. Manual dispatch and worker use identical service logic.
38. Global pause preserves Outbox jobs and does not mark them failed.

### Cursor/incremental sync
39. Cursor scoped by provider/account/resource/repository/query shape.
40. Page fetch has no DB transaction open.
41. Entity upserts + page cursor advance are atomic.
42. Crash before commit does not advance cursor.
43. Restart resumes opaque continuation.
44. Final page clears continuation and advances watermark/last_success_at.
45. Partial/failure stores incomplete reason and does not advance success watermark.
46. Overlap refetch causes no duplicate local Issues.
47. Filter/sort/API query change invalidates cursor.
48. Private/missing 404 never silently deletes local Issue.
49. Absence from a page is not inferred deletion.
50. Manual full sync can cancel and resume safely.

### UI/regression
51. API Monitor shows state from actual worker runtime.
52. Background enable/disable does not toggle global REST pause.
53. Outbox/queue/cursor counts match SQLite.
54. RU/EN UI Gates verify labels/controls and reject placeholder claims.
55. Existing Issues, PR, Review, CI, repository, manual import continue working.
56. Offline local Issue create/edit/project/search/backup remain functional.

## 13. Verification commands and CI

Discover exact scripts first. At minimum run:
- cargo fmt --all --check
- cargo clippy -p forgedesk-core --all-targets -- -D warnings
- cargo test -p forgedesk-core
- npm run typecheck
- npm test
- npm run build
- optional AI crate checks if shared workspace/features are affected;
- Tauri Linux and Windows compile checks;
- relevant Playwright visual UI Gates, including RU/EN API Monitor and Sync settings.

Run focused tests during implementation. Run full relevant CI at each stage boundary and once against the exact final PR head. Report exact SHA and Actions URL. Never infer CI success from an older commit.

Manual smoke test should use a test GitHub repository: connect, pause/resume, queue a linked Issue update offline, reconnect, observe automatic dispatch, test ambiguity via mock fixture, restart mid-pagination, verify UI remains responsive. Do not exhaust real quota or use destructive production Issues.

## 14. Documentation and Wiki

Update after each stage:
- docs/PROJECT_STATE.md — concise current truth (target 1,500–2,000 tokens; no contradictory current claims).
- docs/stages/STAGE-P0-<letter>-report.md — scope, changed paths, exact SHA, commands/results, limits and next stage.
- docs/architecture/overview.md and repository_map.md only when architecture/module layout changes.
- docs/decisions/ADR-0008-background-sync-runtime.md (or next free ADR number) — runtime owner, lock boundaries, source-of-truth and lifecycle.
- 03_GITHUB_API_BUDGET_AND_SYNC.md — separate shipped PR #12 foundation from pending/implemented worker capabilities.
- docs/PRODUCT_GAPS_AUDIT.md — remove claims that PR #12 dispatcher/pause are not implemented.
- GitHub Wiki Home plus dedicated P0-Background-Sync page, concise and linked to this canonical spec.

Never claim Wiki was updated unless the Wiki page was pushed and re-read. If the environment cannot access Wiki repository, commit exact Markdown page source under docs/wiki/ and report Wiki push as BLOCKED, not DONE.

## 15. Mandatory DeepSeek token efficiency

04_TOKEN_EFFICIENCY_RULES.md applies to every call. Additionally:
1. Keep warm prompt stable; no dynamic dates/logs/SHAs before the static prefix.
2. Read current PROJECT_STATE once at start. Do not re-read all six documents or the full repo every iteration.
3. Inspect only files/callers/tests needed for the current stage.
4. Work on one coherent vertical stage per call. Do not re-analyse architecture on each retry.
5. Use minimal unified diffs; do not print full files or successful build logs.
6. Reuse existing types, repositories, components, dependencies and migrations. No second HTTP client/queue/scheduler.
7. Keep same static document order between calls for prompt caching. Put changing task and file excerpts after static context.
8. Do not resend chat history. Never include tokens, private remote payloads or complete user Issues in prompts/logs.
9. Keep PROJECT_STATE around 1,500–2,000 tokens; detailed outcomes belong in stage reports.
10. Track input/output/cache hit/miss/reasoning usage per stage where exposed. Never reduce test/quality scope for token savings.
11. Reuse successful test results only when tested source/tests/environment have not changed. Run required final CI.
12. Each stage response format:

~~~
STATUS: DONE | PARTIAL | BLOCKED
SCOPE: one line
CHANGED: paths + short reasons
TESTS: command — PASS/FAIL/SKIPPED + reason
REVISION: exact SHA
RISKS: only real outstanding risks
NEXT: one concrete action
~~~

## 16. PR and branch rules

- Start from current main and verify SHA.
- Use coherent branches such as feat/p0-sync-runtime-a, feat/p0-sync-budget-b, feat/p0-outbox-worker-c, feat/p0-incremental-cursors-d.
- Keep each PR reviewable and buildable.
- PR description must list scope, migration impact, ambiguity safety, budget behavior, tests and limitations.
- Do not merge, push directly to main, delete branches, force-push/reset or change repository settings. Leave merge decision to user.
- PR #12 is already merged; treat it as baseline, do not recreate it.

## 17. DeepSeek starter prompt

Use after the stable warm prompt from 05_DEEPSEEK_PROMPTS.md and the six canonical documents in their existing order. Attach this specification; do not paste it repeatedly into every call.

~~~
Продолжай ForgeDesk. Реализуй оставшийся P0 строго по 07_P0_BACKGROUND_SYNC_IMPLEMENTATION.md.

BASELINE:
- Repository: mindwork64/forgedesk.
- Start from current main. At task preparation main was e380c58098878080b7d746f3cad7889effdc2fb6 after PR #12; re-check before coding.
- PR #12 already provides shared GitHub REST gate, telemetry/rate observations and functional API Monitor Pause/Resume. Do not rebuild it.
- Schema at that baseline is v11. Re-check CURRENT_SCHEMA_VERSION before migrations. Never edit a published migration.

EXECUTION:
1. Read latest docs/PROJECT_STATE.md, this specification and only relevant files. Apply 04_TOKEN_EFFICIENCY_RULES.md in every call.
2. Implement Stages A–E as buildable vertical slices. Start with Stage A; do not stop after a plan or scaffolding. Continue through P0 unless a real blocker exists.
3. Reuse github_dispatcher.rs, GitHubClient, api_queue_job, sync_cursor, sync_outbox, sync_conflict, AppService and current UI. No second HTTP client, queue or scheduler.
4. Keep Outbox mutation state separate from queue execution. Never retry outcome_unknown without reconciliation. Do not add comments, Issue creation or other remote mutations.
5. Use persistent cursor + transactional page application/watermark. Never advance last_success_at on partial/failed sync.
6. Enforce 10% per-reset-window background allowance, 20% reserve, stale-budget zero allowance, account scope, priority and primary/secondary rate limits.
7. Use deterministic clocks and local mock HTTP server. No destructive live GitHub writes and no real quota exhaustion tests.
8. Update project state, stage reports, architecture/ADR, API budget doc, product gap audit and Wiki page source. Claim Wiki update only after actual push/readback.
9. Run focused checks, then full CI-equivalent checks against final PR head. Report SHA and Actions URL.
10. Create feature branch(es) and PR(s) to main. Do not merge or push directly to main.

Do not conduct a long interview. Choose safe defaults. Ask only for a real blocker involving data loss, security, permissions, external mutations or a material architecture change.

At each stage return STATUS/SCOPE/CHANGED/TESTS/REVISION/RISKS/NEXT. Do not dump full files or successful logs.
~~~

## 18. P0 completion checklist

- [ ] One app-owned worker starts/stops cleanly on Windows/Linux.
- [ ] AppState lock boundaries safe; no lock/transaction spans network.
- [ ] Dispatcher is shared, class-aware, principal-scoped and priority-aware.
- [ ] Single in-flight and mutation pacing enforced.
- [ ] Queue persisted, deduplicated, leased and crash-recoverable.
- [ ] Background consumption <=10% per reset window; 20% reserve protected.
- [ ] Unknown/stale observations block background work; manual pause blocks all authenticated REST.
- [ ] Primary/secondary rate behavior tested.
- [ ] Existing update_issue Outbox rows dispatch automatically when eligible.
- [ ] Ambiguous mutations reconcile before retry; no blind replay.
- [ ] Cursor/page continuation/watermark persisted and transactional.
- [ ] Incremental overlap, query-shape invalidation and restart resume tested.
- [ ] Sync settings/API Monitor show real runtime state RU/EN.
- [ ] CI passes against exact PR head; Windows/Linux compile gates green.
- [ ] Docs and Wiki snapshot accurate.
- [ ] All PRs remain unmerged pending user review.

P0 is complete only when every applicable checklist item is verified.
