# PROJECT_STATE

**Updated:** 2026-11-18  
**Main:** `e380c58098878080b7d746f3cad7889effdc2fb6`  
**Schema:** v11  
**Current objective:** finish the remaining P0 GitHub synchronization runtime according to `docs/implementation/P0_REMAINDER_DEEPSEEK_TZ.md`.

## Verified baseline

PR #12 — [shared GitHub REST dispatcher and API pause gate](https://github.com/mindwork64/forgedesk/pull/12) — is merged. Its tested head was `405f3d4cf4080a49f154d7d29eea53bade4857b4`; [GitHub Actions run 44166546258](https://github.com/mindwork64/forgedesk/actions/runs/44166546258) passed all 7 jobs.

## Implemented

- Local-first desktop app: Rust Stable + Tauri 2 + Vue 3/TypeScript + SQLite, Windows/Linux targets.
- GitHub App Device Flow, token refresh, OS keyring with session-only fallback, repository discovery and bounded/manual Issue import.
- Manual bidirectional updates for already linked GitHub Issues (title/body/state/labels), persisted idempotent Outbox, ambiguous-outcome handling, reconciliation and conflict resolution.
- Shared single-flight authenticated **REST** dispatcher through `GitHubClient`, mutation pacing, request telemetry, REST rate-limit observations and API Monitor Pause/Resume enforcement.
- Existing PR/check/review read and explicit user-triggered PR action flows remain available.

## Remaining P0

1. Lifecycle-managed background scheduler/worker; no worker runs while the app is closed.
2. Common REST + GraphQL dispatcher, priority queue and shared secondary guard. No dedicated GraphQL client currently exists in main.
3. Enforced per-account/resource adaptive quota: max 10% background allowance, 20% protected reserve, zero background allowance for stale/unknown observations.
4. Persistent incremental Issue/repository page cursors and watermarks; current Issue import remains bounded/manual and `sync_cursor` is not yet the active incremental checkpoint.
5. Automatic safe dispatch/reconciliation of existing `sync_outbox` `update_issue` rows; current Outbox dispatch is manually invoked.
6. Active linked PR/check summary refresh and truthful worker/queue/cursor/Outbox states in API Monitor and Sync settings.

P0 does **not** include automatic Issue creation, comment sync, automatic PR merge/close/review, live GitLab/Gitea sync or automatic full historical import.

## Architecture invariants

- `sync_outbox` is the only source of truth for write intents.
- `api_queue_job` is for scheduled/read jobs; do not duplicate Outbox payloads or create another generic queue.
- `sync_cursor` stores page cursor and watermark separately.
- `api_request_log` is telemetry; `api_budget_observation` stores provider observations; `api_control` stores pause state.
- Never perform network I/O inside a SQLite transaction or hold AppState's global lock during network I/O.
- Never blindly retry an ambiguous mutation; reconcile the exact payload fields first.
- OAuth tokens/headers and private Issue bodies/diffs/comments must not enter logs or job payloads.

## Verification boundary

The green 7/7 CI run above verifies PR #12's REST dispatcher/pause slice only. It does not verify background sync, GraphQL dispatch, cursor recovery, automatic Outbox processing or native interactive runtime behavior. Windows/Linux native smoke tests remain unverified.

## Next step

Give DeepSeek the stable warm prompt from `05_DEEPSEEK_PROMPTS.md` and the starter prompt in
`docs/implementation/P0_REMAINDER_DEEPSEEK_TZ.md`. Implement P0-A through P0-E in focused,
reviewable stages and update this file after every merged stage.

Historical details and older verification records are preserved in
[`PROJECT_HISTORY.md`](PROJECT_HISTORY.md).