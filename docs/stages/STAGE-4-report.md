# Stage 4 report — Local Git, GitHub foundation and API budgets

> Historical baseline below; use this dated addendum for current status.

## Current-state addendum — 2026-11-18 (PR #12)

PR #12 is merged to main as `e380c58098878080b7d746f3cad7889effdc2fb6`. Production `GitHubClient` REST requests use a process-wide single-flight gate; REST mutations are paced at least one second apart; REST response telemetry and rate-limit observations are persisted; primary and secondary REST limit responses are classified separately; and API Monitor Pause/Resume enforces the persisted REST pause flag.

Authenticated `GET /user` uses `GitHubClient`; OAuth Device Flow/token exchange/refresh remain separate auth operations. Actions run [44166546258](https://github.com/mindwork64/forgedesk/actions/runs/44166546258) passed 7/7 jobs at PR head `405f3d4cf4080a49f154d7d29eea53bade4857b4`.

Still open: GraphQL through the same dispatcher, priority scheduling, enforced 10%/20% background admission control, adaptive scheduler, and automatic Outbox/cursor workers. Background Sync must remain Not implemented.

---


**Date:** 2026-11-15  
**Status:** PARTIAL — local Git, the API-budget foundation and GitHub App Device
Flow authentication are implemented; live provider client, discovery and dispatcher
remain open.  
**Schema:** v4

## Scope delivered

### Local Git (safe subset)

- Native `std::process::Command("git")` calls only; no `sh -c`, PowerShell
  concatenation or shell interpolation of user values.
- `GIT_TERMINAL_PROMPT=0`, canonicalized repository path, explicit arguments.
- Git availability/version, repository detection, current branch, porcelain status,
  branch list, recent log, working-tree diff stat.
- Create/switch branch with dirty-worktree guard; branch names validated by
  `git check-ref-format --branch` and additional newline/NUL/leading-dash checks.
- Tests: temporary repos, path with spaces/unicode, dirty guard, branch creation/
  switching, shell-injection-shaped branch rejection.

### Repository registry

- Schema v4 `repository` rows for local clones, optional project link, remote URL,
  default branch and last checked timestamp.
- Add/list/get/remove local repositories; duplicate/path validation; Repositories
  list, add dialog and repository detail with Overview/Branches/Recent commits.
- Dirty/clean status, ahead/behind/upstream, diff summary, branch switching and
  branch creation UI.
- Settings → Git now exposes real availability/version, configured repository count,
  supported operations, refresh and a link to Repositories. The old Stage 4
  placeholder was removed; no second Git implementation was added.

### GitHub connection foundation

- `provider_account` schema stores account identity, host, scopes and status —
  explicitly **no token/secret columns**.
- Integrations screen shows GitHub as primary with read/write permission scope,
  connection state and honest "GitHub App credentials are not configured" state.
- Settings → Integrations links to the existing Integrations screen, and onboarding
  describes GitHub as optional and configurable after setup instead of unavailable.
- GitLab/Gitea are visibly planned for Stage 8; Local Git is real.
- GitHub App Device Flow auth is implemented with the public client ID
  `Iv23liclPWLrC2OpaeXz`: start/poll/slow-down/expiry handling, `/user` validation,
  OS keyring storage with memory-only fallback, token refresh and disconnect.
  Live acquisition + refresh verified separately; no token is stored in SQLite or
  written to logs.
- GitHub repository discovery/selection and the budget-aware provider client remain
  open; Connect is real, but no repository sync is enabled yet.

### API Budget Manager foundation

- `api_budget_observation`, `api_request_log`, `api_queue_job`, `api_control`
  schema with sanitized fields (no Authorization headers/tokens/private payloads).
- Background allowance formula implemented and tested:
  `min(10% × limit, max(0, remaining − 20% × limit))`; unknown limit/remaining
  yields zero allowance.
- Latest observation per family/resource, observed ForgeDesk request counts,
  secondary pause state, sanitized request log, queue jobs, manual pause/resume.
- API Monitor UI: connection/scheduler/dispatcher/last observation/reset, GraphQL
  and REST primary panels, resource buckets, queue, recent requests, estimates
  labels, manual pause/resume, sanitized diagnostics copy.
- Unit tests: allowance bounds, observation validation, persistence, request counts,
  pause state. Rate-limit fixture-based UI data is build-gated in the harness only.

## Verification

| Check | Command | Result |
|---|---|---|
| Core tests | `cargo test -p forgedesk-core` | PASS — 44 |
| Core clippy | `cargo clippy -p forgedesk-core --all-targets -- -D warnings` | PASS |
| Format | `cargo fmt --all --check` | PASS |
| Shell check/clippy | `cargo check/clippy -p forgedesk` (pkg-config shims) | PASS typecheck; native link unverified |
| Frontend typecheck | `npm run typecheck` | PASS |
| Frontend tests | `npm test` | PASS — 78 |
| Frontend build | `npm run build` | PASS (chunk-size warning at ~501 kB) |
| UI visual gates | `node scripts/visual-gate0.mjs` | PASS — 43 captures (16 Gate 0 + 13 Gate 1 + 9 Gate 2 + 5 Gate 3) |
| Jenkins | `ForgeDesk-CI` build 7 (Stage 4 snapshot `9ab85bb`) | SUCCESS — 104 s, 44 core tests, 43 captures, 44 artifacts |
| Native Tauri run | — | SKIPPED (no WebKitGTK/sudo) |

## Acceptance mapping

| Stage 4 criterion | Status |
|---|---|
| Safe Local Git operations | PASS subset — detect/status/branches/log/diff/create/switch |
| Dirty guard before branch switch/create | PASS backend tests + UI |
| Unusual path/unicode, no shell injection | PASS tests |
| Local repository attach/list/detail | PASS |
| GitHub App auth/credentials | PASS — Device Flow + keyring/memory vault; live token/refresh test |
| GitHub repository discovery/selection | NOT implemented — auth exists, discovery client pending |
| API Budget Manager model/formula | PASS |
| Persisted observation/request log/queue/control | PASS foundation |
| Serialized live dispatcher/scheduler | NOT implemented — no network provider yet |
| API Monitor screen | PASS for delivered state; live data needs auth |
| Rate-limit fixtures/tests | PASS formula/persistence; live header parsing NOT implemented |
| Offline behavior | PASS — all delivered paths local; connection clearly unavailable |

## Known limitations

- Live HTTP client, GraphQL/REST adapters, ETag/cache, pagination and provider
  repository discovery are not built. Token revocation on the GitHub side is not
  automated; Disconnect deletes the local keyring entry.
- No provider repository discovery; only user-attached local clones.
- No live dispatcher worker; queue persistence/UI exists but no provider request is
  enqueued or executed.
- API Monitor production starts with no observations and shows the empty state;
  connected/budget data in screenshots is visual-harness-only.
- API request log is not populated by a network layer yet; no header parser. The
  displayed observed-request count is the retained log count per bucket, not yet an
  exact reset-window calculation.
- Local Git has no process timeout, open-folder/terminal, branch delete, rebase or
  remote sync; the safe subset is intentionally small.
- Repository detail only has Overview/Branches/Recent commits; PR/issue/release
  tabs depend on later stages.
- No API budget settings UI (fixed documented defaults 10%/20%, max in-flight 1).
- Bundle exceeds Vite's 500 kB warning threshold after Stage 3/4 surfaces.

## Next

Stage 5 — GitHub Issue synchronization requires Stage 4 authentication/network
foundation first. Next concrete slice: GitHub App device-flow connection with
secure token storage, live rate-limit header/GraphQL rateLimit parsing, serialized
dispatcher worker and repository discovery.
