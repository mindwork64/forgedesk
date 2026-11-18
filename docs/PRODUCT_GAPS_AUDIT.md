# ForgeDesk product gaps audit

## Current-state addendum — 2026-11-18 (after PR #12)

This addendum supersedes the older snapshots below. Historical notes are retained as history, not as current implementation status.

- **PR #12 merged** to `main` as `e380c58098878080b7d746f3cad7889effdc2fb6`. PR head `405f3d4cf4080a49f154d7d29eea53bade4857b4`; CI run [44166546258](https://github.com/mindwork64/forgedesk/actions/runs/44166546258) passed 7/7 jobs.
- **Delivered:** process-wide single-flight GitHub REST gate for production `GitHubClient`, one-second mutation pacing, REST telemetry, primary/secondary rate-limit classification, persisted pause enforcement before authenticated REST requests and functional API Monitor Pause/Resume (EN/RU). Authenticated `GET /user` uses `GitHubClient`; Device Flow/token exchange/refresh remain separate auth operations.
- **Not delivered:** GraphQL transport; REST+GraphQL priority scheduler; active 10% background allowance / 20% reserve enforcement; adaptive periodic scheduler; automatic Outbox worker; cursor/watermark-driven incremental Issue sync.
- **Current behavior:** manual GitHub Issue import and manual bidirectional updates for already-linked Issues (title, body, open/closed state, labels) exist. Outbox/conflict models and reconciliation foundations exist, but Outbox dispatch still requires a manual command. `sync_cursor` is persisted but does not yet drive incremental import.
- **API Monitor boundary:** REST dispatcher is Active and REST pause is enforced. Background Sync is not active/implemented. `api_queue_job` is not yet a complete priority execution queue.
- **P0 remaining:** (1) unified REST+GraphQL dispatch and priorities; (2) enforced per-bucket budgets and adaptive scheduler; (3) automatic durable Outbox worker; (4) persistent incremental cursor/watermark sync. See [P0 remaining sync specification](specs/P0_REMAINING_SYNC_SPEC.md).
- **Additional backlog outside this P0 handoff:** Issue creation from local-only tasks, comment synchronization, full historical sync, broader PR/check polling, live GitLab/Gitea support and release hardening.
- **Verification boundary:** CI does not replace interactive Windows/Linux smoke tests or live-provider write verification. PR #12 does not claim either.

The rest of this audit is a historical snapshot. Do not treat its old PR #8/#10/#11 status or “dispatcher not implemented” statements as current.

---

# ForgeDesk product gaps audit

# Current-state addendum — 2026-11-17 (after PR #8 and PR #10)

This addendum supersedes any older statements below that describe PR #8 as pending or say that bidirectional GitHub Issue sync is not implemented. The historical audit body records the earlier PR #2 snapshot and is retained for context.

- **PR #8 is merged** (merge commit `e9c1e4e64b421716d45e74b4cc3bf9ac050e79f4`). Issues #5, #6 and #7 are closed. Project archive/restore, workspace and per-project issue-key prefixes, and the associated search improvements are on `main`. The database schema is v11.
- **PR #10 is merged** (main commit `bd80833002cee660b8f2ef9e262c0556ab4f725a`). Manual bidirectional sync is implemented for already linked GitHub Issues: title, description/body, open/closed status and labels. Outgoing changes use the Outbox; ambiguous writes are reconciled conservatively; field conflicts can be resolved from Settings → Sync.
- **Still not implemented:** background polling/scheduling, a shared dispatcher for all GitHub API requests, a fully enforced API pause/rate-limit gate, automatic creation of GitHub Issues from local-only tasks, comment synchronization and a complete incremental watermark/cursor workflow. Issue import is manually paginated.
- **Still limited:** GitLab/Gitea are fixture-backed/read-only rather than live connections; PR inline review-comment placement and review-thread resolution are missing; local Git has no remote synchronization/rebase/branch deletion or process timeout; workspace export/import beyond SQLite backup/restore and signed installers are not implemented.
- **Verification boundary:** CI and deterministic UI tests do not replace interactive Windows/Linux smoke tests. The PR #11 UX/status corrections are in a draft PR and are not part of `main` yet.

The rest of this file is a historical snapshot produced before those merges; use the addendum above for current status.

---


**Audit date:** 2026-11-17  
**Audited revision:** `4ea91ebf3410e8712f9772e3112efdeb38fc4f91` (main, PR #2 merge)  
**Method:** static inspection of the source tree, routes, UI copy, core services, database repositories and stage reports, followed by CI verification of the implementation in PR #2. This is not a native desktop smoke test.

## Summary

ForgeDesk has substantial working local functionality and a real GitHub integration. The Search and Activity routes and the corrected Sync settings screen are implemented on `main` via PR #2. ForgeDesk is still not a complete end-to-end synchronization product: the shared API dispatcher, background worker and bidirectional sync are not wired.

The highest-risk mismatch was stale Sync settings copy claiming GitHub integration did not exist. The merged status surface accurately labels synchronization as manual-only and explains the instrumentation limitations.

## User-visible placeholder routes

| Route | Current behavior on main | Remaining boundary |
|---|---|---|
| `/search` | Full local workspace search: Issues through SQLite FTS5; Projects; Documents; Repositories. Results link to their owning screens. | Local records only. GitHub issues must first be imported. Search does not query remote GitHub or comments. Initiative name/description and Project description matching are added in pending PR #8, not yet in main. Result sets are bounded. |
| `/activity` | Dedicated read-only timeline of the latest 300 issue activity events, including notifications archived in Inbox. Local issue events and linked GitHub PR status events are classified separately. Entries open the relevant issue. | Not a universal event bus: it does not yet include project/document/repository changes, arbitrary provider webhook events or background sync events that are not stored in `activity_event`. |
| `/settings/sync` | Replaced with actual GitHub connection, manual-only mode, API-monitor snapshot and explicit background-worker limitations. | The UI reports missing sync infrastructure; it does not implement the dispatcher or scheduler. |

## Pending PR #8

https://github.com/mindwork64/forgedesk/pull/8

The branch adds soft project Archive/Restore (Issue #5), editable workspace and
per-project issue-key prefixes plus append-only schema v11 (Issue #7), and two
Search improvements recovered from obsolete PR #3 (Initiative name/description
and Project description). Existing issue keys remain unchanged; all keys use a
single monotonic workspace counter. The existing main Activity implementation
is deliberately untouched. Treat PR #8 as pending until CI/review and merge.

## Synchronization and API budget

### Implemented

- GitHub App Device Flow, token refresh and secure credential storage with memory-only fallback.
- GitHub repository discovery with bounded pagination.
- Manual, bounded GitHub Issue import with idempotent remote links and explicit project targeting.
- PR listing/detail, commits, reviews and check data; destructive or externally mutating PR operations require explicit user intent.
- SQLite schema and repositories for API budget observations, request-log rows, queue rows and pause state.
- REST rate-limit observations are recorded during manual Issue import.

### Not implemented or not wired end-to-end

- No periodic background poller or adaptive synchronization scheduler.
- No shared GitHub request dispatcher. Current client modules issue HTTP requests directly.
- API request log and API queue are read by the monitor, but the current production request path does not populate/execute them as a general dispatcher would.
- The persisted pause flag is not enforced by a shared request gate; the UI control has been removed until that gate exists.
- The outbox state machine and cursors are a persistence/safety foundation; no worker consumes the outbox and reconciles remote writes.
- No bidirectional Issue create/update/comment synchronization, full conflict-resolution workflow or user-facing sync status controls.

**Consequence:** API Monitor is currently a partial diagnostic surface, not evidence that background synchronization is running. Empty request-log/queue panels are expected with the current wiring and must not be described as live dispatcher telemetry.

## Other known incomplete product areas

The stage reports document these deliberately incomplete areas; they should remain visible in the product backlog and must not be described as complete:

- **Issues / Stage 2:** issue templates, watchers, attachments and milestone/cycle fields; workflow transition editor; Activity event edit/delete; Saved View column/density application; UI for `in`/`notIn` filters. The command palette remains a small quick-search/navigation surface; the new Search page is the full local entity search.
- **Issue-key configuration:** global workspace prefix and optional per-project prefix are implemented on the PR #8 branch only; not yet merged into main.
- **Planning / Stage 3:** templates; document import, outline and in-document search; cycle board and burnup/velocity analytics; roadmap milestone level, drag editing and dependency overlays; project-scoped quick-create prefill. Cycle assignment and summary data do exist. Project Archive/Restore UI is proposed in pending PR #8; it is not in main until merge.
- **Local Git / Stage 4:** process timeout, open-folder/terminal, branch deletion, rebase and remote sync.
- **PR / Stage 6:** inline review-comment placement and review-thread resolution remain incomplete.
- **Inbox / Stage 7:** triage routing/assignment and real-time webhooks are explicitly out of v1 scope.
- **GitLab / Gitea / Stage 8:** adapters are fixture-backed/read-only; live host credentials and live-provider verification are not part of the current delivered slice.
- **Release / Stage 10:** native interactive Windows/Linux runtime smoke tests, signed installers, workspace export/import beyond SQLite backup/restore, large-workspace soak tests and performance profiling remain open.

## Documentation drift

- `src/i18n/ru.ts` and `src/i18n/en.ts` described issue keys and workflow as future Stage 2 work even though local Issues/Workflow are implemented.
- The old Sync settings text said GitHub and API monitoring did not exist; it was corrected in PR #2.
- `docs/stages/STAGE-4-report.md` and `docs/stages/STAGE-5-report.md` contain historical acceptance/next-step statements that predate the later discovery/import/PR implementation. They need a dated addendum rather than silent rewriting of historical results.
- `docs/stages/STAGE-10-report.md` contains pre-merge feature-branch statements and old CI evidence. It needs a dated post-merge addendum.
- `docs/PROJECT_STATE.md` should be the single current snapshot; historical stage reports should be explicitly labeled as snapshots.

## Recommended order

1. **Completed in PR #2:** implement Search and Activity, correct Sync status, remove the non-functional pause action, and pass CI/component/visual checks.
2. Implement a shared GitHub HTTP dispatcher and route every network call through rate-limit accounting and an enforced pause gate.
3. Implement the outbox worker, cursor-based sync, reconciliation and conflict UI as a tested vertical slice; only then enable scheduled background polling.
4. Resolve the remaining Stage 2/3 and PR UI gaps according to release scope.
5. Refresh current-state documentation and add dated post-stage updates without changing historical test evidence.

## Verification boundary

The implementation merged in PR #2 was verified by GitHub Actions run
[44004060587](https://github.com/mindwork64/forgedesk/actions/runs/44004060587)
at tested code revision `5d51b33`: all seven CI jobs passed, including 147 frontend
tests, 61 deterministic visual captures, core tests/Clippy on Ubuntu and Windows,
Tauri compile checks on Linux and Windows and the optional AI crate checks.

This remains CI/fixture verification, not a native desktop smoke test. Interactive
Windows/Linux runtime flows and live GitHub credential operations must still be
checked separately.
