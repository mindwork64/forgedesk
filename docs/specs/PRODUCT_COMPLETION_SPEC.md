# ForgeDesk — полное ТЗ на завершение продукта

**Назначение:** мастер-ТЗ для DeepSeek API на завершение всего ForgeDesk. Это не P0-only handoff и не отдельное ТЗ только на GitHub Sync.

**Baseline:** main e380c58098878080b7d746f3cad7889effdc2fb6 (2026-11-18), SQLite schema v11.  
**Нормативные документы:** 01_PRODUCT_PROJECT.md, 02_TECHNICAL_SPEC.md, 03_GITHUB_API_BUDGET_AND_SYNC.md, 04_TOKEN_EFFICIENCY_RULES.md, 05_DEEPSEEK_PROMPTS.md, 06_UI_UX_SPEC.md.

---

## 1. Цель

Довести существующий ForgeDesk до законченного, проверяемого desktop-продукта для одного разработчика на Windows и Linux. Не создавать новый проект рядом, не переписывать уже работающие подсистемы и не ограничиваться P0.

Под «завершить ForgeDesk» здесь понимается:
- закрыть все обязательные P0/P1 и основной P2 scope из SRS и Product Spec;
- довести до конца сквозные workflows: Issue → project/cycle → local Git branch → GitHub Issue/PR → review/checks → merge → task completion;
- закрыть продуктовые хвосты Stage 2–10, описанные ниже;
- довести дополнительную поддержку GitLab/Gitea и optional AI до заявленного уровня, сохранив их необязательность для базового Core;
- закрыть native Windows/Linux, data export/import, reliability, performance, security, accessibility и release readiness.

**Нормативная иерархия:** подтверждённые пользовательские решения → 01_PRODUCT_PROJECT.md → 02_TECHNICAL_SPEC.md → 03_GITHUB_API_BUDGET_AND_SYNC.md → 06_UI_UX_SPEC.md → принятые ADR/фактическая архитектура. 04 и 05 обязательны для каждого DeepSeek API вызова.

ТЗ не разрешает менять продуктовый scope, UI identity, удалять рабочие функции, выполнять необратимые remote actions или вливать PR без прямого разрешения пользователя.

## 2. Актуальная исходная точка

### 2.1. Уже влито в main

- PR #8: Project Archive/Restore, workspace/project issue-key prefixes, Search improvements.
- PR #10: manual bidirectional sync for already-linked GitHub Issues (title/body/open-closed/labels), durable Outbox foundation, reconciliation and conflict resolution.
- PR #11: GitHub connection status messaging and PR detail-pane lifecycle fix.
- PR #12: process-wide single-flight dispatcher for production GitHub REST calls, >=1s mutation pacing, REST telemetry, primary/secondary REST limit classification and enforced API Monitor Pause/Resume.
- PR #12 CI run 44166546258: **7/7 jobs passed** at PR head 405f3d4cf4080a49f154d7d29eea53bade4857b4. Merge commit: e380c58098878080b7d746f3cad7889effdc2fb6.

PR #12 is REST-only. It does not implement GraphQL dispatch or the background sync scheduler. OAuth Device Flow/token exchange/refresh are separate auth operations; authenticated GET /user uses the REST client.

### 2.2. Уже существует в коде

Local shell/Core/SQLite, Settings/backup/restore, RU/EN and themes; Issues CRUD/workflow/statuses/labels/relations/sub-issues/comments/activity, List/Board, FTS5, filters and Saved Views; Projects/Initiatives/Milestones/Cycles/Documents/Roadmap; Local Git safe subset; GitHub App auth, repository discovery/import, PR/reviews/checks/files/actions; manual linked-Issue sync with Outbox/conflicts; Inbox, basic Automations, Insights/Dashboards; fixture-based GitLab/Gitea read-only adapters; optional AI crate/feature with context preview, approval-gated local apply and read-only MCP.

### 2.3. Verification limitations

- Current implementation reports and older stage reports contain stale/contradictory statements. Use this baseline, current source and tests; do not repeat old PR #8/#10/#11 claims as pending.
- PR #12’s green CI is not a native interactive desktop smoke test.
- GitLab/Gitea mock-server tests are not live-host verification.
- AI mock provider tests are not live provider verification.
- At audit time there were no open GitHub Issues; that does not mean the product backlog is empty. This document is the consolidated backlog.

## 3. Architecture constraints

1. Local-first, single-user, one local workspace for v1. SQLite is the source of truth for local entities.
2. App starts and supports local work without network or connected provider.
3. No backend/server, PostgreSQL/MySQL, Redis, Docker/Kubernetes, hosted DB, permanent daemon, mandatory telemetry or cloud queue.
4. Vue renderer uses typed Tauri IPC only; no arbitrary filesystem/network/SQL/shell access. Business logic and permission checks live in Rust Core.
5. No HTTP, pacing/sleep or long work while holding global AppState lock or SQLite write transaction.
6. SQLite migrations are append-only. Never modify migrations 0001–0011; changes require migration 0012+ with defaults, rollback-safe behavior, data-preservation, FK and integrity tests.
7. Tokens stay in OS credential store. Never put tokens, Authorization headers, private payloads or repository secrets into DB logs, diagnostics, screenshots or exports.
8. External mutations require explicit user action or previously approved automation policy. No automatic push/merge or destructive remote actions.
9. Unknown mutation outcome means outcome_unknown; reconcile before any retry. No blind replay.
10. REST and GraphQL share a principal-aware dispatcher/secondary guard; primary REST resources and GraphQL points remain separate budgets.
11. Unsupported capabilities must be explicit; no mock-backed UI presented as a real feature.
12. UI follows 06_UI_UX_SPEC.md: exact geometry/tokens/navigation, RU/EN, Dark/Light, keyboard, accessibility, loading/empty/error/offline/stale/pending and screenshot gates.
13. Background workers are local to the running application, persisted, cancellable and gracefully stopped.
14. Work in small logical PRs. Never merge a PR or publish release without direct user approval.

## 4. Full remaining functionality — source-to-code backlog

DeepSeek must verify every item against current code/tests before changing it. If already completed, cite exact files/tests and mark DONE; do not reimplement.

### 4.1. Issues / Workflow / Views (Stage 2)

**Delivered:** CRUD, FD keys, priorities/types/status categories, labels, relations/sub-issues, comments/activity, archive/duplicate/batch, FTS5, List/Board, basic filters/sort/group, Saved Views persistence/pinning.

**Remaining:**
- Issue templates: CRUD, safe apply to create form, no implicit task creation.
- Single-user watchers: local watch/favorite flag and filters; no multi-user or notification server.
- Managed local attachments: add/open/remove/link, file metadata, safe filenames/path, size/type validation, missing-file state. Backups and workspace export must include managed attachments.
- Complete Issue ↔ Milestone and Issue ↔ Cycle UI/filters/detail flow. Existing link tables must be reused.
- Workflow editor: statuses, category, semantic color, ordering, default status, safe delete/remap behavior. No arbitrary HEX or invalid states.
- Saved View column visibility/order and compact/standard density must actually affect List/Table, not merely be stored.
- Filter UI for in/notIn and parity with backend operators; AND/OR, empty values, dates and relations. Invalid filter fields must show validation, not disappear silently.
- Table, Calendar and Timeline views from 01; List/Board already exist. All views use the same query/filter/sort model and do not duplicate issues.
- Local comment edit/soft-delete where author permits; retain audit integrity. Activity journal remains append-only unless an explicit safe product rule exists.
- Board keyboard alternative and complete keyboard/accessibility audit.

**Acceptance:** template fills a form but requires Submit; attachments survive restart/backup/export/import and resist traversal/collisions; watchers remain local; milestone/cycle links appear in detail/filter screens; workflow edits preserve FK/data; Saved View columns/density actually apply and restore; in/notIn match backend; Table/Calendar/Timeline show the same issue entities; comment updates are transactional and auditable.

### 4.2. Projects / Initiatives / Cycles / Documents / Roadmap (Stage 3)

**Delivered:** Projects/Initiatives/Milestones/Cycles, basic assignments/summaries, Documents CRUD/autosave/Markdown export, project/initiative links, Projects/Initiatives Roadmap and progress aggregation.

**Remaining:**
- Project templates with description/status/priority/progress method/basic milestones; transactional creation and preview.
- Complete progress methods from 01: count-based, estimate-based, milestone-based and manual. Document formulas and test archived/deleted/canceled/double-count edge cases.
- Cycle Board separate from Issue Board; status columns, assignment and carry-over.
- Velocity, burnup and burndown analytics with deterministic formulas and data-through/completeness labels.
- Roadmap milestone level, drag editing and dependency overlays. Validate dependencies/cycles before persistence.
- Project-scoped quick-create: Create Issue from Project Overview must prefill project_id.
- Documents: revision history, templates, Markdown import, outline/heading navigation, in-document search, image/attachment support and links to Issue/Project/Initiative/Repository.
- Import must preview/validate and never overwrite without confirmation. Export/import should round-trip supported Markdown features.
- Project/Initiative/Cycle/Document screens must have real loading/empty/error/offline states and meet UI Gate 2.

**Acceptance:** each progress mode matches test cases; templates create no orphan records; cycle board and analytics reproduce expected fixtures; roadmap changes persist and reject invalid dependencies; document revision can be viewed/restored; Markdown import/export round-trip; outline/search work on large documents; project quick-create links the new Issue; attachments survive backup/export.

### 4.3. Local Git / Repository workflow (Stage 4)

**Delivered:** git availability/version, repo registry, status, branch list/create/switch, log/diff summary, dirty guard, canonical paths, direct argv and injection tests.

**Remaining:**
- Subprocess timeout/cancellation, bounded output, safe child-process shutdown.
- Open folder/terminal/IDE via OS-specific allow-listed executable/arguments; no shell concatenation.
- Branch deletion with current-branch/merged checks; force delete is a separate dangerous action.
- Rebase and safe remote workflow.
- Fetch/pull/push/merge/reset/stash only as explicit commands with dirty guard, preview/state and confirmation. Never auto-push/auto-merge.
- Observable upstream/ahead/behind/last operation/errors/cancel/timeout. Do not invent a full merge-conflict editor outside approved scope.
- Test unusual paths/Unicode, detached HEAD, missing upstream, auth prompt disabled, repo lock, network failure, large output.

**Acceptance:** no subprocess hangs indefinitely; folder/terminal/IDE open or gives an explicit unsupported reason; delete/rebase/remote operations preserve dirty data and require confirmation; no user value is interpolated into a shell command.

### 4.4. GitHub API / API Budget / Dispatcher / Scheduler (Stage 4/5)

**Delivered:** Device Flow/keyring/memory fallback, REST client, PR #12 single-flight REST gate, REST pacing/telemetry/rate classification/pause, repository discovery/import and API Monitor foundation.

**Remaining:**
- GraphQL through the same principal-aware dispatcher and shared single-flight slot/secondary guard; no independent provider HTTP clients.
- Query registry with operation name, fields, pagination, predicted node/cost, permissions, freshness/cache and retry/reconciliation policy.
- GraphQL HTTP 200 + errors, partial data, rate/resource limits, permission/not found and malformed responses classified separately.
- Record actual GraphQL rateLimit.cost/remaining/resetAt; do not add a separate rate probe to every request.
- REST ETag/Last-Modified/304 conditional requests and persisted cache metadata; REST API version centralized and rechecked before release.
- Real priority queue: interactive writes/reads, active context, background sync, manual bulk sync, reconciliation. Fairness/no starvation, cancellation and deadlines.
- REST resource buckets and GraphQL points separate; secondary pause shared. Enforce background allowance = min(10% of observed limit, max(0, observed remaining − 20% reserve)). Unknown/stale observation = zero background allowance.
- Adaptive scheduler with persisted due times, jitter/backoff, max in-flight=1, >=1s mutation spacing. No mass sync at startup.
- Lifecycle: start after auth; stop/delay on pause/offline/401/disconnect/app exit; restore safe jobs after restart.
- Bounded retry for safe reads/background; respect Retry-After/reset; secondary without Retry-After waits >=60s then backoff+jitter. Permission 403 is not rate limit. Mutation timeout after possible send => outcome_unknown.
- API Monitor shows real worker/queue state, next due, separate REST/GraphQL budget, observed-vs-estimated values, background usage/reserve, GraphQL cost stats when samples exist, pause reason and sanitized diagnostics.

**Acceptance:** offline dispatcher tests prove REST+GraphQL single-flight/priority/fairness/pacing/pause/cancel/shutdown; REST and GraphQL errors correctly classified; quotas never merged; stale/unknown budgets block background; interactive action remains usable; no network under AppState/DB locks; no secret/private payload in request log.

### 4.5. GitHub Issues and sync (Stage 5)

**Delivered:** manual bounded import; remote links/ownership; manual outbound update for linked title/body/state/labels; persisted Outbox/conflict models; reconciliation foundation and conflict UI.

**Remaining:**
- Full Issue lifecycle: list/detail/import/link/unlink, create remote Issue from a local Issue by explicit user action, update, labels, milestones, comments and close/archive semantics.
- Comment import/create/update/delete where supported. Persist remote IDs/URLs/timestamps/authors. Reconcile ambiguous comment creation; no blind retries.
- Local Issues remain local unless user explicitly publishes them.
- Field ownership rules: remote title/body/state/labels/comments; local project/cycle/estimate/relations/doc links and other local-only data per ADR. Validate ownership map.
- Automatic durable Outbox worker: atomic claim/lease, ordering per Issue, persisted retry schedule, attempt counters, restart recovery, cancellation and unsupported operation visibility.
- Operation-specific reconciliation for create Issue/comment/state-label update/review/merge. Matching title/body alone is not absolute proof.
- Incremental sync per account/repo/resource/query contract: opaque page cursor distinct from updated_at watermark, overlap, stable sort, transactional page apply+checkpoint, loop detection, resume after restart and filter invalidation.
- Never auto-delete a local Issue because remote is missing/inaccessible. Preserve incomplete/tombstone reason and ask for explicit action.
- Full historical sync is manual, cancellable, resumable and cost/page-estimated.
- Conflict UI covers per-field local/remote/base values and timestamps; previous watermark retained until resolution.
- Sync status badges and per-entity/repo manual refresh.
- GitHub Projects V2 minimum discovery/read/link/basic supported operations in a separate scoped track after Issue sync.

**Acceptance:** repeated import/update is idempotent; remote creation requires explicit action and Outbox record before network; ambiguous mutation reconciles before retry; conflicting edits create durable conflict; cursor commits atomically with page; comments do not duplicate; disconnected/offline state does not block local work; no full sync on startup.

### 4.6. Pull Requests / Reviews / CI / release metadata (Stage 6)

**Delivered:** PR list/detail/commits/reviews/checks/files/diff, review queue/CI screens, review comments list, PR↔Issue links, permission-checked and confirmed review/merge/close actions.

**Remaining:**
- Inline review-comment placement on exact diff lines, old/new side, outdated diff context and path/line mapping.
- Thread replies and resolved/unresolved states where API supports them; explicit Unsupported otherwise.
- GraphQL list summaries for review/check status without heavy N+1 calls; lazy files/diff/comments on demand.
- All PR/Reviews/CI reads through dispatcher and API budget logging.
- Persisted repository selection and multi-repository context as required by product scope.
- Pagination and freshness/offline/access errors for checks/reviews/commits.
- Branch/tag/release metadata and release-published events.
- Merge must pin expected head SHA; stale head rejects and reloads. No blind retry.
- PR labels/assignees/milestones/merge queue are recorded in old Stage 6 notes but are not mandatory in 01–06; keep optional unless scope is explicitly approved.

**Acceptance:** comments anchor to correct diff lines; threads show resolution; no N+1 request explosion; API Monitor logs all requests; stale merge is rejected safely.

### 4.7. Inbox / Triage / Automations / Insights / Dashboards (Stage 7)

**Delivered:** local activity Inbox, read/archive/filter, linked PR state events on refresh, basic local triggers/actions, Automations preview/history, local Insights and editable 12-column Dashboard.

**Remaining:**
- Separate Triage queue for incoming GitHub Issues/local incoming tasks: accept, reject, mark/merge duplicate, select project/priority/labels, create/link Issue. No multi-user assignee model.
- Normalize supported polling events: GitHub Issue opened/updated/comment, PR opened/ready/merged, review requested/changes requested/submitted, CI failed/succeeded, release published, sync conflict/failure and due date reached.
- Deduplicate by stable provider identity/object/timestamp; no webhook relay.
- Full automation triggers/conditions AND/OR/actions for local and supported provider events. Actions include local status/priority/labels, activity, inbox and project/cycle aggregation. Remote actions require explicit policy and permission.
- Idempotency per event/rule, loop protection, preview, execution history, retry/failure state.
- Due-date trigger scheduler with deterministic missed-event catch-up after restart; no polling when app is closed.
- Insights: lead time, backlog age, velocity, completion rate, project progress, PR lead time, review time, CI success in addition to existing summary/throughput/status/project.
- Metric formulas, time ranges, data-through timestamp and completeness must be visible. Provider metrics use synced cache only.
- Dashboard widget registry/configuration for product metrics/recent entities/API budget; add/reorder/remove/resize/configure and persist.

**Acceptance:** triage is usable without another user/assignee; duplicate events create one Inbox row and one automation run; no recursion; PR merge updates linked Issue once; metrics match deterministic fixtures and report missing data; widgets persist; no fake charts.

### 4.8. GitLab/Gitea optional providers (Stage 8)

**Delivered:** provider-neutral adapter contract and fixture/mock-server read-only adapters for GitLab/Gitea repositories/Issues/MRs with pagination and explicit capabilities.

**Remaining for full scope in 01:**
- Secure host/account connection UI, token storage in OS keyring, scopes/status/reconnect/disconnect, self-hosted host URL.
- Persist provider identities without secrets in SQLite.
- Discovery; repositories, Issues/MRs, branches/commits, labels/comments/releases/pipelines/checks where supported.
- Implement writes only where capability and permission are real; reuse Outbox/reconciliation safety.
- Provider-specific pagination/rate-limit/auth/error handling.
- Live test against one GitLab and one Gitea/Forgejo host when available. Without credentials/host, mark NOT VERIFIED and do not overstate.

**Acceptance:** mock contract suite, capability matrix and live tests where test hosts exist; unsupported stays explicit; disconnect stops jobs and clears credentials safely.

### 4.9. Optional AI Extension (Stage 9)

**Delivered:** separate crate/feature, off by default, OpenAI-compatible/Anthropic adapters, context preview, approval-gated append, read-only MCP STDIO.

**Remaining:**
- Product generators: plans, subtasks, acceptance criteria and PR descriptions.
- Structured output validation against domain models; malformed output never partially applies.
- Model discovery, streaming and token accounting where provider supports it; distinguish estimate from actual.
- Explicit context selection and exact preview of payload sent; no secrets or unrelated workspace data.
- Approval for every local mutation and any remote action; normal AppService/provider permission/Outbox paths only.
- Provider errors, timeout, cancellation, invalid keys, usage/cost display and privacy copy.
- Keep MCP explicit opt-in and read-only unless separately approved. Default build/runtime stays AI-free.
- Live provider smoke tests when credentials supplied; fixture tests remain mandatory.

**Acceptance:** default build has no AI dependency/navigation/requests/key load; AI-enabled runtime off has no activity; sent payload matches preview; structured output validates; every mutation is approved.

### 4.10. Release/stabilization (Stage 10)

**Delivered:** GitHub Actions CI lanes, migration tests, axe checks, route code splitting and non-interactive Windows Debug build evidence. PR #12 CI passed 7/7.

**Remaining:**
- Interactive native Windows 10/11 test (WebView2): install, launch, update/uninstall and full user journey.
- Interactive native Linux test (WebKitGTK 4.1) and intended package format.
- Full smoke on both: onboarding, issue/project/document, Git, GitHub auth/import/sync, PR/review/checks, merge→task done, disconnect, offline restart, backup/restore.
- Workspace export/import beyond SQLite backup: versioned archive, manifest/checksum, DB+managed attachments, path validation, preview, compatibility checks and rollback. Never export credentials.
- Realistic schema v11→current and older-supported-schema upgrades. Preserve all entities, links, cursors/outbox/conflicts/documents/attachments; run FK/integrity.
- Large-workspace performance/soak: tens of thousands of issues, many projects/docs/attachments/remote links, long sync, forced exit/restart. Measure startup/query latency/memory/UI responsiveness/DB growth.
- Local structured error logs and sanitized diagnostics; no crash upload/backend/telemetry by default. Recover interrupted worker/outbox state.
- Signed MSI/packages when credentials exist. Without cert, unsigned artifacts/checksums clearly labeled; signing remains external blocker, not fake PASS.
- User documentation: install/update/uninstall, data locations, backup/restore, export/import, GitHub permissions, providers, offline behavior, troubleshooting.
- Dependency/security review, keyboard/accessibility, Light/Dark parity, production harness exclusion and final release metadata.

**Acceptance:** native Windows+Linux user journeys with evidence; export/import round-trip including attachments and safe rollback; migration/integrity tests; measured performance; production excludes harness and default AI; signing state honest.

## 5. Явно вне scope без отдельного решения

- Multi-user/roles/presence/team assignments, billing, SAML/SCIM.
- Backend, hosted DB, Redis, webhook relay, permanent daemon.
- Real-time webhooks; use adaptive polling/on-demand.
- Auto-push/auto-merge/destructive remote operations.
- Full mirror of every repository or all remote history by default.
- Embedded IDE/code editor, mandatory AI, mandatory telemetry or crash upload.
- PR labels/assignees/milestones/merge queue unless promoted by explicit approval.

Triage assignment means project/priority/labels, not assigning to another user.

## 6. Порядок реализации

| Track | Scope | Dependency | Exit gate |
|---|---|---|---|
| 0 | One-time source-to-requirement audit/traceability matrix | none | every item DONE/PARTIAL/MISSING/NOT VERIFIED |
| 1 | Issues/Workflow/Views | 0 | Stage 2 + UI Gate 1 |
| 2 | Planning/Documents/Roadmap | 1 | Stage 3 + UI Gate 2 |
| 3 | Local Git | 0 | safe Git contract + Windows/Linux tests |
| 4 | REST+GraphQL dispatcher/budget/scheduler/cache | 0 | deterministic API budget suite |
| 5 | GitHub Issue lifecycle/Outbox/cursors/Projects V2 base | 4 | Stage 5 end-to-end DoD |
| 6 | PR/Reviews/CI and metadata | 4,5 | Stage 6 complete |
| 7 | Inbox/Triage/Automations/Insights/Dashboards | 5,6 | issue→PR→merge→completion |
| 8 | GitLab/Gitea live capability adapters | 4,5 | contract + live/NOT VERIFIED report |
| 9 | AI optional extension | core stable | default-off + optional feature gates |
| 10 | Native release/export/performance/security/docs | all required tracks | full product DoD |

Do not put the whole backlog in one PR. Tracks 1–3 may be separate; do not enable large background sync before Track 4/5 safety gates.

## 7. Глобальная Definition of Done

ForgeDesk complete only when:
- all mandatory requirements from 01–06 have traceability and PASS evidence;
- local Issues/Projects/Cycles/Documents/Views work offline and persist across restart;
- GitHub uses real API, separate REST/GraphQL budgets, safe mutations, incremental sync and visible conflicts;
- PR/Reviews/CI link correctly to local Issues; end-to-end merge completion works;
- GitLab/Gitea capabilities are live-tested or explicitly NOT VERIFIED/unsupported;
- optional AI can be built/enabled but default Core remains AI-free;
- backup/restore and workspace export/import include managed attachments;
- no data loss/duplicate remote writes/false live states;
- UI Gates 0–5, Dark/Light, RU/EN, keyboard/accessibility and screenshot evidence pass;
- GitHub-hosted Actions full matrix green on exact release candidate SHA;
- native interactive Windows and Linux smoke evidence exists separately from compile CI;
- install/update/uninstall and user docs are ready; known external blockers (e.g. signing cert) are explicit.

## 8. Test matrix

All new behavior must have deterministic offline tests:
- DB: migration preservation, FK/integrity, backup/restore/export/import, attachments and rollback.
- Issues: templates/watchers/attachments/relations/cycles/workflow transitions/comments/filter operators/Saved View columns/density.
- Planning: progress formulas/templates/cycle carry-over and analytics/roadmap dependencies/document revisions/import/export/search.
- Git: unusual paths/Unicode/injection/dirty state/timeout/cancel/detached HEAD/upstream/remote failure/no data loss.
- GitHub API: REST headers/ETag/304/403/429; GraphQL cost/errors/partial data; primary/secondary; Retry-After/reset; permission/auth failures.
- Dispatcher: REST+GraphQL single-flight, priority/fairness, pacing, pause, cancellation/shutdown.
- Sync: Outbox idempotency/claim/recovery/reconciliation, conflict, comments duplicate prevention, cursor atomicity/overlap/restart/filter invalidation.
- PR: stale SHA, inline comment line mapping/thread state, pagination, budget logging.
- Productivity: all automation triggers/actions, idempotency/loop protection/due catch-up, metrics/dashboard fixtures.
- Providers: GitLab/Gitea mock contract + live tests when hosts/credentials available.
- AI: default off/no key load/no network, preview equals payload, structured validation, approvals and cancellation.
- UI: component flows and screenshot evidence for every modified screen; final Gate 5 audit.
- Soak: large workspace plus long-running background sync and forced restart.

Live tests must be opt-in and use dedicated test repositories/accounts; never run destructive live tests against user's real repositories.

## 9. Required CI/manual checks

Use existing repo conventions; required lanes include:
- cargo fmt --all --check
- cargo test -p forgedesk-core
- cargo clippy -p forgedesk-core --all-targets -- -D warnings
- cargo test/clippy for forgedesk-ai optional crate
- cargo check -p forgedesk and feature ai-extension where supported
- npm run typecheck, npm test, npm run build
- VITE_AI_EXTENSION=true npm run build
- node scripts/visual-gate0.mjs / scripts/ci.sh full gate
- GitHub Actions GitHub-hosted matrix: Core Ubuntu+Windows, Frontend, Optional AI, Tauri Linux+Windows, UI visual gates.

Do not assume self-hosted CI runners. If native runtime is unavailable, mark SKIPPED and provide a reproducible user smoke checklist; do not call compile PASS a native PASS.

## 10. Documentation deliverables

Keep current:
- docs/PROJECT_STATE.md — compact truth, target ~1,500–2,000 tokens.
- docs/PRODUCT_GAPS_AUDIT.md — full gap matrix.
- docs/architecture/overview.md + repository_map.md — current code map.
- docs/decisions/ADR-xxxx.md — real design decisions only.
- docs/stages/STAGE-N-report.md — delivered scope and honest test evidence.
- docs/wiki/Home.md, Architecture.md, Project-Stages.md, Product-Roadmap.md — same truth as repo docs.
- docs/release/RELEASE_CHECKLIST.md, README.md, CHANGELOG.md.
- docs/specs/PRODUCT_COMPLETION_SPEC.md — this master plan.
- docs/specs/P0_REMAINING_SYNC_SPEC.md — one subsystem sub-spec only.

Do not rewrite historical test evidence silently; add dated addenda. Every PASS claim includes exact SHA and command/CI run.

## 11. Token efficiency and DeepSeek workflow

Every call follows 04_TOKEN_EFFICIENCY_RULES.md:
- Keep existing warm prompt from 05_DEEPSEEK_PROMPTS.md unchanged and first in stable prefix.
- Stable document order: 01, 02, 03, 04, 05, 06, master spec, compact PROJECT_STATE.
- Stage 0 creates the repository map once; later calls read only 3–8 directly related files/tests/ADR.
- Use minimal unified diffs, not regenerated full files; filter logs; do not repeat successful tests without changed inputs.
- Keep cache prefix stable; changing state/logs go after static docs.
- Keep PROJECT_STATE compact and stage reports factual.
- Track input/output/cache-hit/cache-miss/reasoning/model/duration/cost when available; do not send secrets to analytics.
- Save tokens on repetition, never on security, correctness or required tests.

## 12. Warm and starter prompt

**Warm prompt:** use the existing unchanged warm prompt in 05_DEEPSEEK_PROMPTS.md. Do not fork/rewrite it.

**Starter prompt — whole ForgeDesk:**

Начни полное завершение ForgeDesk по PRODUCT_COMPLETION_SPEC.md, а не только P0 или GitHub Sync.

1. Прочитай актуальный PROJECT_STATE и мастер-план; сверяй scope с 01–06.
2. Выполни одноразовый Stage 0 audit по фактическому коду, UI, migrations and tests. Старые stage reports не считать доказательством без проверки.
3. Создай docs/architecture/product-traceability.md: requirement → files/current behavior → DONE/PARTIAL/MISSING/NOT VERIFIED → evidence/test → next action.
4. Обнови PROJECT_STATE, PRODUCT_GAPS_AUDIT, architecture/repository map, stage reports and wiki mirror to current truth.
5. Предложи sequence of small logical PRs using Tracks 1–10; no giant PR.
6. На Stage 0 не делай массового refactor. После audit выбери первую ближайшую самостоятельную задачу и открой отдельный implementation PR.
7. На каждом track добавляй tests, RU/EN UI states, docs, stage report and exact CI evidence.
8. Не задавай вопросов о внутренних деталях. Выбирай простой безопасный вариант и фиксируй ADR; спрашивай лишь при реальном блокере/риске потери данных/необратимой remote операции/смене permissions/существенном scope conflict.
9. Не выполняй merge, force push, release или destructive remote action без моего прямого разрешения.
10. Fixture tests ≠ live tests; compile CI ≠ native runtime; PARTIAL ≠ DONE.

Формат результата:
STATUS: DONE | PARTIAL | BLOCKED
SCOPE: ...
CHANGED: ...
TESTS: command — PASS/FAIL/SKIPPED
EVIDENCE: exact SHA/CI/screenshot/live run
RISKS: only real blockers
NEXT: one concrete task

Выполняй последовательно до полного product DoD. Не объявляй продукт завершённым по одному зелёному CI или закрытию P0.

## 13. Финальный критерий

Полная готовность — весь обязательный продуктовый scope 01–06 реализован и проверен end-to-end, все optional tracks имеют честный статус, Windows/Linux native smoke evidence присутствует, export/import и release checklist закрыты. P0 completion alone is not product completion.
