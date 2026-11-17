# ForgeDesk product gaps audit

**Audit date:** 2026-11-17  
**Audited revision:** `5b68247e292d4a7f81029aba9f208c745de10f3f` (main)  
**Method:** static inspection of the source tree, routes, UI copy, core services, database repositories and stage reports, followed by CI verification of the implementation in PR #2. This is not a native desktop smoke test.

## Summary

ForgeDesk has substantial working local functionality and a real GitHub integration. The exposed Search and Activity routes and the stale Sync settings screen are now implemented on the current PR branch. ForgeDesk is still not a complete end-to-end synchronization product: the shared API dispatcher, background worker and bidirectional sync are not wired.

The highest-risk mismatch was stale Sync settings copy claiming GitHub integration did not exist. The current branch replaces it with a status surface that accurately labels synchronization as manual-only and explains the instrumentation limitations.

## User-visible placeholder routes

| Route | Current behavior on this branch | Remaining boundary |
|---|---|---|
| `/search` | Full local workspace search: Issues through the SQLite FTS5 query path; Projects through the project query; Documents by title/body; Repositories by name, description, remote URL and checkout path. Results link to their owning screens. | Local records only. GitHub issues must first be imported. Search does not query remote GitHub, comments, cycles or initiatives. Result sets are bounded (up to 40 issues and 12 per other entity group). |
| `/activity` | Dedicated read-only timeline of the latest 300 issue activity events, including notifications archived in Inbox. Local issue events and linked GitHub PR status events are classified separately. Entries open the relevant issue. | Not a universal event bus: it does not yet include project/document/repository changes, arbitrary provider webhook events or background sync events that are not stored in `activity_event`. |
| `/settings/sync` | Replaced with actual GitHub connection, manual-only mode, API-monitor snapshot and explicit background-worker limitations. | The UI reports missing sync infrastructure; it does not implement the dispatcher or scheduler. |

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
- The persisted pause flag is not enforced by a shared request gate.
- The outbox state machine and cursors are a persistence/safety foundation; no worker consumes the outbox and reconciles remote writes.
- No bidirectional Issue create/update/comment synchronization, full conflict-resolution workflow or user-facing sync status controls.

**Consequence:** API Monitor is currently a partial diagnostic surface, not evidence that background synchronization is running. Empty request-log/queue panels are expected with the current wiring and must not be described as live dispatcher telemetry.

## Other known incomplete product areas

The stage reports document these deliberately incomplete areas; they should remain visible in the product backlog and must not be described as complete:

- **Issues / Stage 2:** issue templates, watchers, attachments and milestone/cycle fields; workflow transition editor; Activity event edit/delete; Saved View column/density application; UI for `in`/`notIn` filters. The command palette remains a small quick-search/navigation surface; the new Search page is the full local entity search.
- **Planning / Stage 3:** templates; document import, outline and in-document search; cycle board and burnup/velocity analytics; roadmap milestone level, drag editing and dependency overlays; project archive menu; project-scoped quick-create prefill. Cycle assignment and summary data do exist.
- **Local Git / Stage 4:** process timeout, open-folder/terminal, branch deletion, rebase and remote sync.
- **PR / Stage 6:** inline review-comment placement and review-thread resolution remain incomplete.
- **Inbox / Stage 7:** triage routing/assignment and real-time webhooks are explicitly out of v1 scope.
- **GitLab / Gitea / Stage 8:** adapters are fixture-backed/read-only; live host credentials and live-provider verification are not part of the current delivered slice.
- **Release / Stage 10:** native interactive Windows/Linux runtime smoke tests, signed installers, workspace export/import beyond SQLite backup/restore, large-workspace soak tests and performance profiling remain open.

## Documentation drift

- `src/i18n/ru.ts` and `src/i18n/en.ts` described issue keys and workflow as future Stage 2 work even though local Issues/Workflow are implemented.
- The old Sync settings text said GitHub and API monitoring did not exist; it is corrected in this change.
- `docs/stages/STAGE-4-report.md` and `docs/stages/STAGE-5-report.md` contain historical acceptance/next-step statements that predate the later discovery/import/PR implementation. They need a dated addendum rather than silent rewriting of historical results.
- `docs/stages/STAGE-10-report.md` contains pre-merge feature-branch statements and old CI evidence. It needs a dated post-merge addendum.
- `docs/PROJECT_STATE.md` should be the single current snapshot; historical stage reports should be explicitly labeled as snapshots.

## Recommended order

1. Finish CI, component tests and visual checks for the Search/Activity/Sync settings changes.
2. Implement a shared GitHub HTTP dispatcher and route every network call through rate-limit accounting and an enforced pause gate.
3. Implement the outbox worker, cursor-based sync, reconciliation and conflict UI as a tested vertical slice; only then enable scheduled background polling.
4. Resolve the remaining Stage 2/3 and PR UI gaps according to release scope.
5. Refresh current-state documentation and add dated post-stage updates without changing historical test evidence.

## Verification boundary

The implementation in PR #2 was verified by GitHub Actions run
[44002363339](https://github.com/mindwork64/forgedesk/actions/runs/44002363339)
at tested code revision `8ef6c62`: all seven CI jobs passed, including 147 frontend
tests, 61 deterministic visual captures, core tests/Clippy on Ubuntu and Windows,
Tauri compile checks on Linux and Windows and the optional AI crate checks.

This remains CI/fixture verification, not a native desktop smoke test. Interactive
Windows/Linux runtime flows and live GitHub credential operations must still be
checked separately.
