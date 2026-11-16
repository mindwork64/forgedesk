# PROJECT_STATE

**Updated:** 2026-11-16
**Stage:** 10 — Repository model v10 implemented; native/signing and sync writes remain partial
**Verified CI revision:** `7a1e892` on branch `fix/repository-project-associations`; feature branch CI run `43858188104` passed all jobs. Not merged to main.

## Current objective

Validate the repository/checkout/project model end-to-end and merge the feature
branch after CI. Remaining PR polish, native packaging and Stage 5 sync writes stay
tracked.

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
- **Tests:** 99 core (+2 ignored live), 5 AI crate, 128 frontend, 59 default visual captures (+1 optional AI capture).

### Stage 5 — sync safety foundation and read-only import

- **Schema v10:** logical `repository` (provider identity, remote-only), `repository_checkout`, `repository_project`, plus v9 sync/inbox/automation/dashboard tables. Legacy `repository.project_id` is migrated and no longer read.
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

- **Schema v7/v8:** `inbox_state` stores read/archived state; `automation_rule` and `automation_run` store local rules/history.
- Inbox lists local issue activity (created, updated, status/labels, comments, archive,
  relations) with source badges, Today/Yesterday/Earlier groups, All/Unread/GitHub/
  Local/Sync filters, unread markers, mark read/unread, archive and mark-all-read.
- Normalization currently covers local `activity_event` rows; GitHub sync events will
  be added with the sync engine. No fake external events.
- Detail pane shows event summary/entity and links to local issues.
- **Insights:** local aggregates for created/completed/open/overdue, average cycle
  time, 12-bucket completion throughput, status and project distribution, with
  `data through`; range 30/90/365 days.
- **Automations v1:** local trigger→label rules, enabled toggle, preview, idempotent
  run with per-event dedupe, run history and last result. No remote writes.
- **Dashboards:** persisted 12-column layout, default metric/recent-issues/API-budget
  widgets, edit mode with add/remove/width/reorder; local data only.
- **External PR events:** on PR list refresh ForgeDesk syncs linked PR states; changes
  are deduped and written as `pr_merged`/`pr_state_changed` activity, visible in Inbox
  as GitHub-source events.
- **End-to-end completion:** merging a linked PR marks the local issue Done, records
  `pr_merged` activity and updates the link state; live-tested on the test repository.
- Triage routing/assignment and real-time webhooks remain out of scope for v1.

### Stage 8 — GitLab/Gitea adapters (complete, fixtures)

- `ProviderAdapter` contract: normalized repositories/issues/merge requests plus
  explicit read-only capabilities; no fake provider names as capability evidence.
- GitLab: projects/issues/MRs, `PRIVATE-TOKEN`, `X-Next-Page`.
- Gitea: user repos/issues/pulls, token auth, `X-HasMore`, PR filtering from issues.
- Integrations cards show read-only adapter ready, credentials not configured and
  supported capability list.
- Contract tests use local mock HTTP servers; live hosts/tokens are unavailable and
  explicitly reported as not run.

### UI correction — real Git/GitHub settings exposure

- Settings → Git is now a functional screen: real availability/version, repository
  count, supported operations, refresh and a link to Repositories.
- Settings → Integrations points to the existing Integrations screen instead of an
  obsolete Stage 4 placeholder.
- Onboarding describes GitHub as optional and configurable in Integrations after
  setup; no disabled fake Connect action or Stage 4 badge.

### Repository model correction (schema v10)

- Logical `repository` identity (provider/host/opaque external ID/owner/name/metadata)
  supports remote-only repositories without a local path.
- `repository_checkout` stores local clones; a repository may have zero or many
  checkouts. `repository_project` is the many-to-many source of truth.
- v10 migration preserves legacy repository IDs, checkouts, project links, Issues and
  remote links; `remote_issue_link.repository_id` is renamed to
  `remote_repository_key`.
- GitHub discovery upserts repositories and exposes pagination; import fetches outside
  the DB transaction, creates/upserts the repository and project association first,
  and supports explicit target projects and partial import continuation.
- Project Overview Repositories and unified Repositories UI support remote-only
  repositories, multiple projects, link/unlink and checkout attachment.
- PR repository selection is stored by repository ID in SQLite settings.
- Verified locally: 99 core tests, 128 frontend tests, 59 visual captures; feature
  branch CI run `43858188104` passed all seven jobs. Not merged to main.

### Stage 10 — stabilization/release (partial)

- Route-level code splitting: main JS chunk reduced from ~501 KB to ~231 KB; Vite
  chunk warning removed.
- Realistic v5→v10 migration test preserves workflow/statuses, project, issue,
  comment, document, repository, remote link, outbox and cursor data; foreign keys are
  checked.
- axe-core `button-name`/`aria-command-name` validation runs on every visual case,
  with valid/invalid regression fixtures; all 58 cases pass.
- AI crate Clippy (`-D warnings`) is part of the dedicated CI job; the job
  explicitly installs the Clippy component. GitHub Actions run `43778341394` passed
  all jobs.
- CI covers default and optional AI builds; release checklist added.
- GitHub Actions CI run
  [43778341394](https://github.com/mindwork64/forgedesk/actions/runs/43778341394)
  passed all seven jobs at `bddc4c58`: Core Ubuntu/Windows, Frontend, Optional AI
  crate, Tauri Linux/Windows compile and UI visual gates.
- Windows 10 Pro x64 Build 19042 native run
  [43778393711](https://github.com/mindwork64/forgedesk/actions/runs/43778393711)
  passed on a self-hosted runner: frontend typecheck/tests/build, Core tests, Core
  Clippy, formatting, `cargo build -p forgedesk` and
  `cargo check -p forgedesk --features ai-extension`.
  - Debug executable `forgedesk.exe` (7,968,310 bytes):
    [artifact](https://github.com/mindwork64/forgedesk/actions/runs/43778393711/artifacts/14187497885).
  - This is a **Debug executable**, not an MSI/release installer; interactive app
    launch and user scenarios were not exercised.
- The temporary `.github/workflows/win10-runner-smoke.yml` workflow and WIN10 runner
  are not part of normal CI and are removed/deprecated.
- Node.js 20/runtime deprecation warnings appeared during Actions runs but did not
  affect results; no dependency modernization is included in this update.
- Native desktop **interactive** smoke tests, signed installers, export/import and
  larger soak tests remain release blockers.

### Stage 9 — optional AI extension (complete, fixtures)

- Separate `forgedesk-ai` crate + optional `ai-extension` desktop feature; default
  builds do not link it and default UI has no AI nav/DOM.
- OpenAI-compatible and Anthropic adapters with mock HTTP contract tests.
- Explicit context selection/preview, runtime off by default, API key in OS keyring,
  non-secret config JSON.
- AI never mutates directly: applying a suggestion requires confirmation and the
  normal local issue update path.
- `forgedesk-mcp` STDIO binary exposes only read-only `forgedesk_context`.
- No live provider credential was available; live calls are explicitly not run.

### Stages 1–3 (preserved)

- Local-first shell/settings/backups/themes.
- Issues/workflow/labels/relations/comments/search/filters/saved views/board.
- Projects/initiatives/milestones/cycles/documents/roadmap.

## Verification

| Check | Command | Result |
|---|---|---|
| Core tests | `cargo test -p forgedesk-core` | PASS (99 + 2 ignored live) |
| AI crate tests | `cargo test -p forgedesk-ai` | PASS (5) |
| AI feature build | `cargo check -p forgedesk --features ai-extension` | PASS |
| AI frontend build | `VITE_AI_EXTENSION=true npm run build` | PASS |
| Core clippy | `cargo clippy -p forgedesk-core --all-targets -- -D warnings` | PASS |
| Shell check/clippy | `cargo check/clippy -p forgedesk` (pkg-config shims) | PASS typecheck; native link unverified |
| Format | `cargo fmt --all --check` | PASS |
| Frontend typecheck | `npm run typecheck` | PASS |
| Frontend tests | `npm test` | PASS (128) |
| Frontend build | `npm run build` | PASS — main chunk ~231 KB; warning gone |
| Visual gates | `node scripts/visual-gate0.mjs` | PASS — 59 captures; axe button-name/aria-command-name |
| GitHub Actions CI | run `43778341394` at `bddc4c58` | PASS — all 7 jobs |
| Windows 10 native build/tests | run `43778393711` at `bddc4c58` | PASS — Debug `forgedesk.exe` artifact; no interactive run |
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
- GitLab/Gitea adapters are contract-tested but not live-tested; no connection/token
  UI or provider persistence exists yet.
- GitHub repository discovery is live; only the selected repo is used for PRs.
- No dispatcher worker; issue import records response-header budget observations, but
  PR reads and request logs have no producer yet. Observed-request counts are log
  totals, not exact reset-window usage.
- API Monitor production starts disconnected/empty; connected/budget screenshots use
  the build-gated harness.
- Local Git has no process timeout, open folder/terminal, branch delete, rebase or
  remote sync; safe subset only.
- Repository detail has Overview/Branches/Recent commits only.
- GitHub discovery/import live network paths remain mock/fixture-tested; no live
  provider write is performed. Import page continuation is explicit and bounded.
- No API budget settings UI; documented defaults 10%/20%, max in-flight 1.
- Stage 3 follow-ups remain (templates, doc import/outline, cycle analytics, roadmap
  milestones).
- Windows 10 native Debug compilation and automated tests are verified; interactive
  Windows launch/user flows and Linux native runtime are not.
- Signed installers and MSI/release packaging are blocked on signing credentials.
- GitHub Actions emits non-fatal Node.js 20 deprecation warnings; no CI architecture
  changes were made.
- Workspace export/import beyond SQLite backup/restore is not implemented.
- GitHub Actions on the public repo is green at `bddc4c58` (run `43778341394`):
  frontend, core Ubuntu/Windows, Tauri Linux/Windows compile, optional AI crate and
  visual gates. A separate successful Windows 10 native Debug build is documented
  above; it does not replace interactive smoke testing.
- Jenkins mirror requires local git daemon; rotate the API token after setup.

## Next step

Release blockers: interactive Windows/Linux smoke tests, signing credentials/MSI,
export/import, and large-workspace/soak testing. Stage 5 sync writes and live AI/GitLab/Gitea
verification remain tracked. Remaining Stage 6 polish (inline comment
placement, PR read budgets) and Stage 5 sync writes stay tracked.
