# ForgeDesk product gaps audit

**Audit date:** 2026-11-17  
**Audited revision:** `5b68247e292d4a7f81029aba9f208c745de10f3f` (main)  
**Method:** static inspection of the current source tree, routes, UI copy, core services, database repositories and stage reports. This is not a native desktop smoke test.

## Summary

ForgeDesk has substantial working local functionality and a real GitHub integration. It is not yet a complete end-to-end synchronization product. Several screens still expose generic placeholders, and some settings/status copy was not updated after later stages landed.

The highest-risk mismatch is the old Sync settings placeholder: it says GitHub integration does not exist, even though GitHub authentication, repository discovery, manual issue import and PR workflows exist. The page is being replaced by an accurate status surface in the current change.

## User-visible placeholder routes

| Route | Current behavior | Required product decision |
|---|---|---|
| `/search` | Replaced with full-page local search across Issues (SQLite FTS), Projects, Initiatives, Documents and Repositories. Results open their owning screens. | Implemented in the current follow-up; GitHub-wide remote search is intentionally not implied. |
| `/activity` | Replaced with a workspace-wide, read-only feed of recorded Issue events. It includes archived issues and does not inherit Inbox read/archive filtering. | Implemented in the current follow-up; it currently covers recorded Issue events only, not every provider/PR/system event. |
| `/settings/sync` | Was a Stage 4 `SettingsPlaceholder` with copy claiming GitHub had not arrived. | Replaced in this change with actual connection, manual-sync, API-monitor and background-sync status. |

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

- **Issues / Stage 2:** issue templates, watchers, attachments and milestone/cycle fields; workflow transition editor; Activity event edit/delete; Saved View column/density application; UI for `in`/`notIn` filters. The command palette performs local issue search, but the dedicated Search route is still a placeholder.
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

1. Finish review/verification of the Sync settings correction and the Search/Activity pages.
2. Implement the shared request dispatcher and connect every GitHub HTTP path to budget accounting before enabling a background scheduler.
3. Implement the outbox worker, cursor-based manual/background sync, reconciliation and conflict UI as a single tested vertical slice.
4. Resolve the remaining Stage 2/3 and PR UI gaps according to release scope.
5. Refresh current-state documentation and add dated post-stage updates without changing historical test evidence.

## Verification boundary

This audit inspected source and documentation at the audited revision. It did not run a native desktop session or execute the repository's test suite. CI on the pull request must provide the code-change verification; interactive Windows/Linux smoke tests remain separate release checks.
