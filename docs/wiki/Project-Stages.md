# Project stages — full completion status

| Stage | Scope | Status |
|---|---|---|
| 1 | Desktop/Core/SQLite/UI shell | Implemented; native interactive smoke open |
| 2 | Issues/Workflow/Search/Views | Partial: templates, watchers, attachments, workflow editor, saved-view columns/density, in/notIn, more views |
| 3 | Projects/Initiatives/Cycles/Documents/Roadmap | Partial: templates, cycle board/analytics, roadmap milestones/dependencies, document history/import/outline/search |
| 4 | Local Git + GitHub API budget | Partial: REST dispatcher merged; GraphQL, scheduler/budget enforcement and Local Git gaps remain |
| 5 | GitHub Issue sync | Partial: manual import/update; automatic Outbox/cursor workers, comments/create and incremental sync remain |
| 6 | PR/Commits/Checks/Reviews | Mostly implemented; inline comments/thread resolution and complete API budget integration remain |
| 7 | Inbox/Triage/Automations/Insights/Dashboards | Partial: triage, broader triggers/actions and complete metrics/widget set remain |
| 8 | GitLab/Gitea | Fixture/read-only adapters; live account and supported operations remain |
| 9 | Optional AI | Default-off extension exists; generators/streaming/usage accounting and live verification remain |
| 10 | Stabilization/Release | Partial: native smoke, export/import, perf/soak, packaging/signing/docs |

## Completion order

1. One-time requirement-to-source traceability audit.
2. Finish Issues/Workflow/Views.
3. Finish Planning/Documents.
4. Finish Local Git.
5. Complete REST+GraphQL dispatcher, API budget and scheduler.
6. Finish GitHub Issue lifecycle and safe incremental sync.
7. Finish PR/reviews/CI.
8. Finish Inbox/Triage/Automations/Insights/Dashboards.
9. Finish live GitLab/Gitea capabilities.
10. Finish optional AI extension.
11. Native Windows/Linux and release hardening.

PR #12 delivered a REST dispatcher slice only. Green compile CI is not native interactive smoke evidence. See docs/specs/PRODUCT_COMPLETION_SPEC.md for full acceptance.
