# ForgeDesk — актуальный аудит продуктовых пробелов

**Снимок:** 2026-11-18; main e380c58098878080b7d746f3cad7889effdc2fb6; SQLite schema v11.  
**Главный handoff:** docs/specs/PRODUCT_COMPLETION_SPEC.md.

## Уже доставлено

- PR #8: Project Archive/Restore, issue-key prefixes, Search improvements.
- PR #10: manual bidirectional sync for linked GitHub Issues, Outbox/reconciliation and conflict UI.
- PR #11: GitHub connection status and PR detail lifecycle fix.
- PR #12: shared process-wide GitHub REST dispatcher, mutation pacing, REST telemetry/rate classification and enforced API Monitor Pause/Resume.
- PR #12 CI run 44166546258 passed 7/7 jobs at tested PR head 405f3d4cf4080a49f154d7d29eea53bade4857b4; merged as e380c58098878080b7d746f3cad7889effdc2fb6.

## Backlog

| Area | Current state | Remaining |
|---|---|---|
| Issues/Workflow/Views | CRUD, List/Board, filters, Saved View base | templates, watchers, attachments, milestone/cycle UI, workflow editor, saved-view columns/density, in/notIn, comment edit/delete, Table/Calendar/Timeline |
| Planning/Documents | projects/initiatives/milestones/cycles/docs/roadmap base | project templates/progress methods, cycle board/analytics, roadmap milestone/drag/dependencies, docs history/templates/import/outline/search/attachments, scoped quick-create |
| Local Git | safe status/branch/log/diff subset | process timeout/cancel, open folder/terminal/IDE, branch delete/rebase, safe explicit remote ops |
| GitHub API | REST client + REST dispatcher/pause | GraphQL same dispatcher, priority/fairness, ETag/cache, budget enforcement/scheduler, full API Monitor |
| GitHub Issue sync | manual import and linked-field update | create/comments/labels/milestones, durable Outbox worker, safe reconciliation, cursor/watermark incremental sync, ownership/conflicts |
| PR/Reviews/CI | list/detail/files/reviews/checks/actions | inline comments/thread resolution, GraphQL summaries, budget logs, broader repo scope, tags/releases |
| Inbox/Triage | local events, read/archive, linked PR events on refresh | actual triage queue, broader polling, full automation trigger/action set, anti-loop guards |
| Insights/Dashboards | basic local aggregates/widgets | lead time, backlog age, velocity, completion/project progress, PR/review lead times, CI success, full widget registry/config |
| GitLab/Gitea | fixture/read-only adapters | secure live account lifecycle and supported read/write capabilities; live tests |
| Optional AI | default-off extension, context preview, approved append, read-only MCP | plan/subtask/acceptance/PR generators, streaming/model discovery/token accounting, stronger structured validation/live tests |
| Release | CI, migration and accessibility foundation | native Windows/Linux smoke, export/import incl. attachments, performance/soak, final packaging/signing/docs |

## Scope boundaries

- Single-user product: no team/assignee/role model. Triage assignment is project/priority/labels.
- No real-time webhooks or webhook relay; use adaptive polling/on-demand.
- No backend, hosted DB, Redis, mandatory AI or telemetry.
- Never auto-push/auto-merge or execute destructive remote mutations without user intent.
- PR labels/assignees/milestones/merge queue appear in an old Stage 6 backlog but are not mandatory in 01–06; keep optional unless separately approved.
- GitLab/Gitea and AI are secondary optional tracks; their real status must be clear.

## Order

1. Traceability audit against 01–06 and current code/tests.
2. Finish Issues/Workflow/Views.
3. Finish Planning/Documents.
4. Finish Local Git.
5. Finish REST+GraphQL API manager/scheduler.
6. Finish GitHub Issue lifecycle and safe incremental sync.
7. Finish PR/review/CI.
8. Finish Inbox/Triage/Automations/Insights/Dashboards.
9. Finish live GitLab/Gitea.
10. Finish optional AI.
11. Native Windows/Linux, export/import, performance, security/accessibility and release.

Full details, acceptance matrix and DeepSeek starter prompt: docs/specs/PRODUCT_COMPLETION_SPEC.md.
