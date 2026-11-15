# PROJECT_STATE

**Updated:** 2026-11-15  
**Stage:** 5 — GitHub sync foundation (auth done; live sync blocked)  
**Revision:** working tree / local git repository (see `git log -1`)

## Current objective

Implement the live provider client and outbox dispatcher on top of the working
GitHub App authentication. Live issue synchronization is still blocked; no fake sync
or external write is enabled.

## Architecture (locked)

Rust workspace: `crates/forgedesk-core` (domain, SQLite, app services, git process
wrapper, backups; no Tauri) + `src-tauri` (typed Tauri 2 shell). Vue 3 + TS + Vite
renderer with typed IPC only. SQLite WAL, serialized writes, append-only migrations
through `user_version`. See ADR-0001…0006.

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
  core tests include Stage 5 safety coverage.
- **API Monitor:** connection/scheduler/dispatcher/observation/reset summary,
  GraphQL/REST/resource budget panels, secondary state, background allowance/reserve,
  queue, recent sanitized requests, export diagnostics.
- **Integrations:** GitHub primary with real Device Flow auth, readable scopes and
  keyring/memory token state; Local Git real; GitLab/Gitea planned. No token/secret
  schema and no fake connect.
- **Tests:** 52 core (+1 ignored live import), 78 frontend, 45 visual captures.

### Stage 5 — sync safety foundation (backend only)

- **Schema v5:** `remote_issue_link`, `sync_cursor`, `sync_outbox`, `sync_conflict`.
- Remote link ownership JSON, per-resource cursors (page/watermark/last success),
  idempotent outbox enqueue and closed state machine:
  `pending → dispatching → succeeded | failed_definite | outcome_unknown →
  reconciling → succeeded | retry_safe | needs_user_action`.
- Ambiguous outcomes cannot retry directly; tests cover idempotency, attempts,
  invalid transitions, reconciliation path, links, cursors and conflict resolution.
- **GitHub App Device Flow auth:** start/poll/slow-down/expiry, `/user` validation,
  OS keyring storage with memory-only fallback, refresh token and disconnect.
  Live token acquisition + refresh were verified; no token is persisted in SQLite or
  written to logs. Read-only repository discovery + issue import is implemented and
  live-tested; local→remote writes, comments and the outbox dispatcher remain open.

### Stages 1–3 (preserved)

- Local-first shell/settings/backups/themes.
- Issues/workflow/labels/relations/comments/search/filters/saved views/board.
- Projects/initiatives/milestones/cycles/documents/roadmap.

## Verification

| Check | Command | Result |
|---|---|---|
| Core tests | `cargo test -p forgedesk-core` | PASS (52 + 1 ignored live) |
| Core clippy | `cargo clippy -p forgedesk-core --all-targets -- -D warnings` | PASS |
| Shell check/clippy | `cargo check/clippy -p forgedesk` (pkg-config shims) | PASS typecheck; native link unverified |
| Format | `cargo fmt --all --check` | PASS |
| Frontend typecheck | `npm run typecheck` | PASS |
| Frontend tests | `npm test` | PASS (78) |
| Frontend build | `npm run build` | PASS (Vite chunk warning ~501 kB) |
| Visual gates | `node scripts/visual-gate0.mjs` | PASS — 45 captures (16/13/9/7 across Gates 0–3) |
| Jenkins build 6 | Stage 3 `8a999ee` | SUCCESS |
| Jenkins build 7 | Stage 4 snapshot `9ab85bb` | SUCCESS — 104 s, 44 core tests, 43 captures, 44 artifacts |
| Jenkins build 8 | Stage 5 foundation `6d6bf52` | SUCCESS — 109 s, 46 core tests, 43 captures, 44 artifacts |
| Jenkins build 9 | Auth foundation `64f7469` | SUCCESS — 199 s, 49 core tests, 44 captures, 45 artifacts |
| Jenkins build 10 | GitHub discovery/import `5d894e6` | SUCCESS — 124 s, 52 core tests, 45 captures, 46 artifacts |
| Tauri config | `npx tauri info` | Valid; WebKitGTK absent locally |

## Known issues / limitations

- GitHub App auth is implemented, but live HTTP/GraphQL/REST client, ETag/pagination,
  provider repository discovery and outbox dispatcher are NOT implemented.
- Stage 5 has no live issue import/export, label/comment sync, sync UI, conflict
  resolution UI or dispatcher consuming the outbox. Only the safety model, auth and
  tests exist.
- Token storage uses OS keyring; when unavailable it falls back to session-only
  memory and warns. Token revocation is local-only (Disconnect deletes the entry).
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
- GitHub Actions on the public repo is green (frontend, core Ubuntu/Windows, Tauri
  Linux/Windows, visual gates); Linux core now installs `libdbus-1-dev` for keyring.
- Jenkins mirror requires local git daemon; rotate the API token after setup.

## Next step

Live REST/GraphQL client with rate-limit header/`rateLimit` parsing, serialized
dispatcher worker, GitHub repository discovery, then Stage 5 issue import/export and
conflict UI. Authenticate first via Integrations → Connect.
