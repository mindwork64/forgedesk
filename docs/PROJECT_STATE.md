# PROJECT_STATE

**Updated:** 2026-11-15  
**Stage:** 4 — Local Git/GitHub foundation/API budgets (local vertical slice delivered)  
**Revision:** working tree / local git repository (see `git log -1`)

## Current objective

Implement GitHub App authentication +
live provider infrastructure before Stage 5 issue synchronization. GitHub connection
is intentionally not faked in production.

## Architecture (locked)

Rust workspace: `crates/forgedesk-core` (domain, SQLite, app services, git process
wrapper, backups; no Tauri) + `src-tauri` (typed Tauri 2 shell). Vue 3 + TS + Vite
renderer with typed IPC only. SQLite WAL, serialized writes, append-only migrations
through `user_version`. See ADR-0001…0005.

## Implemented

### Stage 4 — local Git and API budget foundation

- **Schema v4:** `repository`, `provider_account`, `api_budget_observation`,
  `api_request_log`, `api_queue_job`, `api_control`.
- **Safe Local Git:** availability/version, repository detection, status, branches,
  log, diff stat, create/switch branch with dirty guard; direct argv `Command`,
  `GIT_TERMINAL_PROMPT=0`, branch validation; tests for spaces/unicode and
  injection-shaped names.
- **Repositories:** local attach/list/detail/remove, project link, Repositories
  table, add dialog, detail Overview/Branches/Recent commits, dirty guard UI.
- **API Budget Manager:** formula `min(10%×limit, max(0, remaining−20%×limit))`,
  persisted observations/requests/queue/control, pause/resume, sanitized diagnostics,
  44 core tests.
- **API Monitor:** connection/scheduler/dispatcher/observation/reset summary,
  GraphQL/REST/resource budget panels, secondary state, background allowance/reserve,
  queue, recent sanitized requests, export diagnostics.
- **Integrations:** GitHub primary with readable scopes and honest unavailable auth;
  Local Git real; GitLab/Gitea planned. No token/secret schema and no fake connect.
- **Tests:** 44 core, 78 frontend, 43 visual captures.

### Stages 1–3 (preserved)

- Local-first shell/settings/backups/themes.
- Issues/workflow/labels/relations/comments/search/filters/saved views/board.
- Projects/initiatives/milestones/cycles/documents/roadmap.

## Verification

| Check | Command | Result |
|---|---|---|
| Core tests | `cargo test -p forgedesk-core` | PASS (44) |
| Core clippy | `cargo clippy -p forgedesk-core --all-targets -- -D warnings` | PASS |
| Shell check/clippy | `cargo check/clippy -p forgedesk` (pkg-config shims) | PASS typecheck; native link unverified |
| Format | `cargo fmt --all --check` | PASS |
| Frontend typecheck | `npm run typecheck` | PASS |
| Frontend tests | `npm test` | PASS (78) |
| Frontend build | `npm run build` | PASS (Vite chunk warning ~501 kB) |
| Visual gates | `node scripts/visual-gate0.mjs` | PASS — 43 captures (16/13/9/5 across Gates 0–3) |
| Jenkins build 6 | Stage 3 `8a999ee` | SUCCESS |
| Jenkins build 7 | Stage 4 snapshot `9ab85bb` | SUCCESS — 104 s, 44 core tests, 43 captures, 44 artifacts |
| Tauri config | `npx tauri info` | Valid; WebKitGTK absent locally |

## Known issues / limitations

- GitHub App auth/device flow, secure token storage, token refresh/revocation and
  live HTTP client are NOT implemented (no app credentials; no fake connection).
- No GitHub repository discovery/selection; only user-attached local clones.
- No dispatcher worker/network adapters/header parser/ETag/pagination; queue and
  request log have no producer in production yet. Observed-request counts are log
  totals, not exact reset-window usage.
- API Monitor production starts disconnected/empty; connected/budget screenshots use
  the build-gated harness.
- Local Git has no process timeout, open folder/terminal, branch delete, rebase or
  remote sync; safe subset only.
- Repository detail has Overview/Branches/Recent commits only.
- No API budget settings UI; documented defaults 10%/20%, max in-flight 1.
- Stage 3 follow-ups remain (templates, doc import/outline, cycle analytics, roadmap
  milestones).
- Bundle exceeds Vite’s 500 kB warning threshold.
- Native Tauri runtime/screenshots remain unverified.
- Jenkins mirror requires local git daemon; rotate the API token after setup.

## Next step

GitHub App Device Flow with secure token storage (keychain/OS credential vault),
live REST/GraphQL client, rate-limit header/`rateLimit` parsing, serialized
dispatcher worker and GitHub repository discovery. Then Stage 5 issue sync.
