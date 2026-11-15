# PROJECT_STATE

**Updated:** 2026-11-15
**Stage:** 7 — Inbox/triage started (partial); Stage 5 sync writes remain partial
**Revision:** working tree / local git repository (see `git log -1`)

## Current objective

Complete the remaining PR polish (inline comment placement, PR read budgets) and
Stage 5 reconciliation. GitHub merge/close/review actions are permission-checked and
require explicit confirmation; no automatic retry of ambiguous writes.

## Architecture (locked)

Rust workspace: `crates/forgedesk-core` (domain, SQLite, app services, git process
wrapper, backups; no Tauri) + `src-tauri` (typed Tauri 2 shell). Vue 3 + TS + Vite
renderer with typed IPC only. SQLite WAL, serialized writes, append-only migrations
through `user_version`. See ADR-0001…0007.

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
- **Tests:** 81 core (+3 ignored live), 107 frontend, 55 visual captures.

### Stage 5 — sync safety foundation and read-only import

- **Schema v7:** `remote_issue_link`, `sync_cursor`, `sync_outbox`, `sync_conflict`, `pull_request_link`, `inbox_state`.
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

### Stage 6 — pull requests, checks and reviews (complete)

- GitHub REST: list pulls (open/closed/all), pull detail, commits, reviews and
  check-runs for the head SHA.
- Discovered GitHub repository can be selected for PRs; selection stored in WebView
  storage (SQLite persistence pending).
- Pull Requests screen: Open/Closed/All filter, refresh, table and 440px detail pane
  with Overview / Commits / Checks / Reviews.
- Overview shows description, branches, additions/deletions/files, mergeability and
  requested reviewers; Checks shows status/conclusion/duration; Reviews shows
  reviewer/state/body; Files shows unified patches with status and counts.
- PR↔local-issue links are stored in schema v6 (`pull_request_link`) with link/unlink
  UI; no GitHub object is modified by linking.
- Permission-aware actions: merge (merge/squash/rebase with expected head SHA),
  close and review submission (approve/request changes/comment) require explicit
  confirmation and server-side permission checks; no auto-retry on ambiguous writes.
- Live read-only verification against `cli/cli` passed for list + detail.
- Disconnect is idempotent, keyring-error-safe and clears memory/pending/warnings
  only after a successful credential delete; PR file pagination loads all pages
  (100/page) with cycle/non-advance protection and whole-request failure on later
  page errors.
- PR Files completeness distinguishes a below-cap incomplete list from files exceeding
  the 3000-file endpoint cap and from unknown completeness at the cap; warnings are
  non-blocking and loaded patches stay visible.
- Credential-storage failures—including Entry creation—use the stable
  `credential_storage` wire code. Only that code shows saved-authorization-unavailable;
  auth/network/internal failures show a separate connection-check error, with retry/
  reauthorization. "No account connected" is rendered only after a successful check
  confirms absence. Invalid saved JSON returns a safe auth error without deletion.
- Reviews queue and CI/Activity screens are read-only and use GitHub search plus
  check-runs; commit selection lazily loads checks. Live read-only verification
  passed for both endpoints. PR review comments are shown under the matching file
  with path/line/author/body. Files support Unified/Split diff modes; inline comment
  placement and thread resolution remain open.
- Split diff/line comments, review submission, merge/close, PR↔issue links and
  budget-aware PR request logging remain open.

### Stage 7 — Inbox/triage (partial)

- **Schema v7:** `inbox_state` stores read/archived state for normalized activity events.
- Inbox lists local issue activity (created, updated, status/labels, comments, archive,
  relations) with source badges, Today/Yesterday/Earlier groups, All/Unread/GitHub/
  Local/Sync filters, unread markers, mark read/unread, archive and mark-all-read.
- Normalization currently covers local `activity_event` rows; GitHub sync events will
  be added with the sync engine. No fake external events.
- Detail pane shows event summary/entity and links to local issues.
- **Insights:** local aggregates for created/completed/open/overdue, average cycle
  time, 12-bucket completion throughput, status and project distribution, with
  `data through`; range 30/90/365 days.
- Automations and Dashboards remain to do.

### Stages 1–3 (preserved)

- Local-first shell/settings/backups/themes.
- Issues/workflow/labels/relations/comments/search/filters/saved views/board.
- Projects/initiatives/milestones/cycles/documents/roadmap.

## Verification

| Check | Command | Result |
|---|---|---|
| Core tests | `cargo test -p forgedesk-core` | PASS (81 + 3 ignored live) |
| Core clippy | `cargo clippy -p forgedesk-core --all-targets -- -D warnings` | PASS |
| Shell check/clippy | `cargo check/clippy -p forgedesk` (pkg-config shims) | PASS typecheck; native link unverified |
| Format | `cargo fmt --all --check` | PASS |
| Frontend typecheck | `npm run typecheck` | PASS |
| Frontend tests | `npm test` | PASS (107) |
| Frontend build | `npm run build` | PASS (Vite chunk warning ~501 kB) |
| Visual gates | `node scripts/visual-gate0.mjs` | PASS — 55 captures (16/13/9/15/2 across Gates 0–4) |
| Jenkins build 6 | Stage 3 `8a999ee` | SUCCESS |
| Jenkins build 7 | Stage 4 snapshot `9ab85bb` | SUCCESS — 104 s, 44 core tests, 43 captures, 44 artifacts |
| Jenkins build 8 | Stage 5 foundation `6d6bf52` | SUCCESS — 109 s, 46 core tests, 43 captures, 44 artifacts |
| Jenkins build 9 | Auth foundation `64f7469` | SUCCESS — 199 s, 49 core tests, 44 captures, 45 artifacts |
| Jenkins build 10 | GitHub discovery/import `5d894e6` | SUCCESS — 124 s, 52 core tests, 45 captures, 46 artifacts |
| Jenkins build 11 | Stage 6 read-only PR slice `ec5e9dc` | SUCCESS — 144 s, 52 core tests (+2 ignored), 47 captures, 48 artifacts |
| Jenkins build 12 | PR files/diff `3dc3ee1` | SUCCESS — 128 s, 52 core tests (+2 ignored), 48 captures, 49 artifacts |
| Jenkins build 13 | GitHub fixes `8ff8ab4` | SUCCESS — 121 s, 63 core tests (+2 ignored), 48 captures, 49 artifacts |
| Tauri config | `npx tauri info` | Valid; WebKitGTK absent locally |

## Known issues / limitations

- Stage 5 read-only issue import exists; label/comment sync, local→remote writes,
  sync UI, conflict resolution UI and the outbox dispatcher remain open.
- Stage 6 write actions are implemented with explicit confirmation and permission
  checks, but have not been executed against a live GitHub repository. Inline
  comment placement/thread resolution, budget-aware PR request logging and native
  Tauri verification remain open; review/check table columns show `—` for rows whose
  detail has not been loaded.
- PR read calls do not yet write API-budget observations/request logs; selected
  repository is WebView storage, not SQLite.
- Token storage uses OS keyring; when unavailable it falls back to session-only
  memory and warns. Token revocation is local-only (Disconnect deletes the entry).
- GitHub repository discovery is live; only the selected repo is used for PRs.
- No dispatcher worker; issue import records response-header budget observations, but
  PR reads and request logs have no producer yet. Observed-request counts are log
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

Continue Stage 7: automations (rules, preview, run history, loop protection), then
Dashboards with persisted 12-column layout. Remaining Stage 6 polish (inline comment
placement, PR read budgets) and Stage 5 sync writes stay tracked.
