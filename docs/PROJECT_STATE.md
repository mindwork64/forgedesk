# PROJECT_STATE

**Current state (2026-11-17, after PR #8 and PR #10):**
- Main commit: `bd80833002cee660b8f2ef9e262c0556ab4f725a`; schema v11.
- PR #8 is merged: project Archive/Restore, workspace and per-project issue-key prefixes, Search improvements. Issues #5, #6 and #7 are closed.
- PR #10 is merged: manual bidirectional sync for already linked GitHub Issues (title, body, open/closed status and labels), Outbox dispatch, conservative reconciliation and field-conflict resolution.
- Sync is still manual. Background polling/scheduling, a shared dispatcher for every GitHub request, fully enforced API pause/rate-limit gate, automatic creation of GitHub Issues from local-only tasks, comment sync and full incremental cursor/watermark are not implemented.
- GitLab/Gitea are fixture-backed/read-only, not live-connected. Native interactive desktop smoke tests, signing, workspace export/import beyond SQLite backups and large-workspace soak tests remain open.
- The PR #11 UX/status corrections are in draft review and are not in `main` yet.

**Historical snapshot:** The original status and pending PR #8 notes below predate PR #8 and PR #10. Retain them as a record of the earlier implementation, not as the current backlog.


**Updated:** 2026-11-17
**Main schema:** v10. **Proposed branch schema:** v11 in PR #8 (pending CI/review; not yet merged).
**Stage:** 10 — Repository model v10 implemented; native/signing and sync writes remain partial
**Verified feature revision:** `5d51b33` (PR #2 code tested by CI run `44004060587`, 7/7 jobs passed). Merged to `main` via PR #2 as `4ea91ebf3410e8712f9772e3112efdeb38fc4f91`.

## Current objective

Search/Activity and Sync-settings corrections are merged to `main` via PR #2.
After PR #8 is reviewed and merged: shared GitHub request dispatcher, enforced API pause gate
and sync worker.

## Current implementation status — 2026-11-17 (PR #2 merged to main)

- **Search page:** `/search` is no longer a generic placeholder. It searches local Issues
  through SQLite FTS5, Projects through the project query, Documents by title/body and
  Repositories by identity/description/remote URL/checkout path. It returns bounded
  result groups and navigates to the owning screen. It does not query remote GitHub
  or unimported issues.
- **Activity page:** `/activity` is no longer a generic placeholder. It shows the latest
  300 stored issue activity events, including items archived from Inbox, and links
  entries back to issues. Linked PR state changes are classified as GitHub events.
  It is not yet a general workspace event bus or a webhook stream.
- **Inbox source classification:** `sync_*` activity kinds are now retained as Sync
  instead of being collapsed into Local; `pr_*` entries are classified as GitHub.
- **Sync settings:** the current page truthfully reports manual-only GitHub access.
  No periodic scheduler, shared HTTP dispatcher, queue worker or outbox reconciler
  is currently wired.
- **Verification:** GitHub Actions run #99
  ([44004060587](https://github.com/mindwork64/forgedesk/actions/runs/44004060587))
  passed all seven jobs at tested implementation revision `5d51b33`: 147 frontend
  tests, UI Gate 0 with 61 captures, core tests/Clippy on Ubuntu and Windows,
  Tauri compile checks on Linux and Windows, and optional AI crate checks. Native
  interactive desktop flows and live GitHub credential flows remain unverified.

## Pending implementation — PR #8

Branch: `feat/project-controls-and-key-prefixes`  
PR: https://github.com/mindwork64/forgedesk/pull/8  
Current branch head at this snapshot: `159c19d` (the PR is open; CI is running).

- **Issue #5:** project detail now exposes Archive/Restore. This is a soft archive;
  issues, documents and repository associations remain in place. Archived projects
  can be found via Projects → Show archived and restored. No hard-delete is added.
- **Issue #7:** Settings → Workspace has an editable global issue-key prefix.
  Project create/edit has an optional override; blank inherits the workspace prefix.
  Prefixes are normalized to uppercase ASCII letters/digits, 1–10 characters, starting
  with a letter. One workspace-wide monotonic counter is shared across projects, so
  changing prefixes never rewrites old keys or causes a collision.
- SQLite migration v11 appends nullable `project.issue_key_prefix`; existing projects
  inherit the workspace prefix after migration.
- Search adds local Initiative name/description search and Project description search.
  Main's richer Activity view is unchanged.
- Tests added for prefix normalization/allocation, immutable old keys, v10→v11 upgrade,
  Search initiatives/navigation, Settings prefix save and archive/restore store behavior.
- These changes are **not in main until PR #8 is merged**. The pending Actions result
  must be checked before calling the branch verified; native interactive runtime tests
  remain a separate check.

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
  persisted observations/requests/queue/control and sanitized diagnostics. The saved
  pause flag is persistence only; it is not an enforced network gate and no pause
  control is exposed until a shared request dispatcher exists.
- **API Monitor:** shows persisted budget observations, request-log/queue rows and
  diagnostics, but these are partial. It explicitly marks the background worker and
  request dispatcher as not implemented and no longer exposes the non-functional
  Pause/Resume action. REST rate-limit observations are currently recorded during
  manual Issue import; other GitHub calls are not routed through a shared dispatcher.
  Request logs/queue are not complete network telemetry, and the saved pause flag
  does not gate direct client calls.
- **Integrations:** GitHub primary with real Device Flow auth, readable scopes and
  keyring/memory token state; Local Git real; GitLab/Gitea planned. No token/secret
  schema and no fake connect.
- **Tests:** 106 core (+2 ignored live), 5 AI crate, 139 frontend, 59 default visual captures (+1 optional AI capture).

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
- Inbox normalizes stored issue `activity_event` rows. Source classification retains
  `sync_*` event kinds as Sync and linked PR events as GitHub; no fake external events.
  The Sync filter now has a reachable classification, but background sync still does
  not produce events because its worker is not implemented.
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

### Search, Activity and Sync settings (PR #2 — merged to main)

- Search/Activity are implemented as route-level lazy-loaded pages with English and
  Russian strings, component tests and visual harness fixtures/cases.
- Search scope is intentionally limited to records already stored in this local
  workspace. It searches up to 40 issue matches and up to 12 matches per other entity
  group; no remote provider search or comment search is claimed.
- Activity reads at most 300 issue-linked events and is independent of Inbox archive
  state. Project/document/repository audit events and full provider/webhook history
  are not implemented.
- Settings → Sync describes manual-only behavior and explicitly warns that the shared
  dispatcher, enforced pause gate, background scheduler and sync worker are missing.

### Repository model correction (schema v10)

- Logical `repository` identity (provider/host/opaque external ID/owner/name/metadata)
  supports remote-only repositories without a local path.
- `repository_checkout` stores one optional local clone per logical repository.
  `repository_project` is the many-to-many source of truth.
- v10 migration preserves legacy repository IDs, checkouts, project links, Issues and
  remote links; `remote_issue_link.repository_id` is renamed to
  `remote_repository_key`.
- GitHub discovery upserts repositories and exposes pagination; import fetches outside
  the DB transaction, creates/upserts the repository and project association first,
  and supports explicit target projects and partial import continuation.
- Project Overview Repositories and unified Repositories UI support remote-only
  repositories, multiple projects, link/unlink and checkout attachment. Multiple
  local clones per remote are intentionally out of scope for v1.
- PR repository selection is stored by repository ID in SQLite settings.
- Verified locally: 106 core tests, 139 frontend tests, 59 visual captures. Post-merge
  `main` CI run `43969772720` passed all seven jobs; Windows Release Installer run
  `43969772719` produced the MSI successfully.

### Stage 10 — stabilization/release (partial)

- Route-level code splitting: main JS chunk reduced from ~501 KB to ~244 KB; Vite
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
| Core tests | `cargo test -p forgedesk-core` | PASS (106 + 2 ignored live) |
| AI crate tests | `cargo test -p forgedesk-ai` | PASS (5) |
| AI feature build | `cargo check -p forgedesk --features ai-extension` | PASS |
| AI frontend build | `VITE_AI_EXTENSION=true npm run build` | PASS |
| Core clippy | `cargo clippy -p forgedesk-core --all-targets -- -D warnings` | PASS |
| Shell check/clippy | `cargo check/clippy -p forgedesk` (pkg-config shims) | PASS typecheck; native link unverified |
| Format | `cargo fmt --all --check` | PASS |
| Frontend typecheck | `npm run typecheck` | PASS |
| Frontend tests | `npm test` | PASS (126) |
| Frontend build | `npm run build` | PASS — main chunk ~244 KB; warning gone |
| Visual gates | `node scripts/visual-gate0.mjs` | PASS — 59 captures; axe button-name/aria-command-name |
| GitHub Actions CI | run `43969772720` at `670b499` | PASS — all 7 jobs |
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
- PR read calls do not yet write API-budget observations/request logs. The selected
  PR repository is persisted by stable repository ID in SQLite settings; localStorage
  remains only a compatibility cache.
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
- GitHub Actions on the public repo is green at `670b499` (run `43969772720`):
  frontend, core Ubuntu/Windows, Tauri Linux/Windows compile, optional AI crate and
  visual gates. A separate successful Windows 10 native Debug build is documented
  above; it does not replace interactive smoke testing.
- Jenkins mirror requires local git daemon; rotate the API token after setup.

## Next step

Release blockers: interactive Windows/Linux smoke tests, signing credentials/MSI,
export/import, and large-workspace/soak testing. Stage 5 sync writes and live AI/GitLab/Gitea
verification remain tracked. Remaining Stage 6 polish (inline comment
placement, PR read budgets) and Stage 5 sync writes stay tracked.
