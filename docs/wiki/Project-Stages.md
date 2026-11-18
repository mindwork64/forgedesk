# Project stages

| Stage | Scope | Current status |
|---|---|---|
| 1 | Desktop foundation, SQLite, settings, backups, UI shell | Complete |
| 2 | Issues, workflow, search, saved views, board | Complete |
| 3 | Projects, initiatives, milestones, cycles, documents, roadmap | Partial; follow-ups remain |
| 4 | Local Git, repositories, API budget and dispatcher | Partial; REST dispatcher/pause landed in PR #12; GraphQL/common budget runtime and scheduler remain |
| 5 | GitHub Issue synchronization | Manual import and linked Issue update/Outbox/conflict foundations exist; automatic worker and incremental cursors remain |
| 6 | Pull requests, commits, checks, reviews | Main functionality implemented; UI/interaction polish remains |
| 7 | Inbox, triage, automations, analytics, dashboards | Partial |
| 8 | GitLab/Gitea providers | Fixture-backed/read-only adapters; live connections not implemented |
| 9 | Optional AI extension | Complete as optional extension |
| 10 | Stabilization and release | Partial; native interactive smoke, signing and soak testing remain |

## PR #12 — merged 2026-11-18

The REST dispatcher, mutation pacing, API telemetry/rate observations and
API Monitor Pause/Resume enforcement are on main. CI run 44166546258 passed all
seven jobs at the exact PR head.

## Remaining P0 — GitHub sync runtime

- lifecycle-managed background scheduler and worker;
- common REST + GraphQL dispatcher, per-account budget and shared secondary guard;
- 10% background cap / 20% protected reserve with zero allowance for stale/unknown observations;
- incremental Issue/repository sync with persisted page cursor and watermark;
- automatic dispatch/reconciliation for existing `update_issue` Outbox records;
- real worker/queue/Outbox/cursor states in API Monitor and Sync settings.

The canonical task breakdown, acceptance criteria and DeepSeek starter prompt are in
[the P0 implementation spec](../implementation/P0_REMAINDER_DEEPSEEK_TZ.md).
