# Project stages

| Stage | Scope | Status |
|---|---|---|
| 1 | Desktop foundation, SQLite, settings, backups, UI shell | Complete |
| 2 | Issues, workflow, search, saved views, board | Complete |
| 3 | Projects, initiatives, milestones, cycles, documents, roadmap | Partial |
| 4 | Local Git, repositories, API budget foundation, API Monitor | Partial — REST dispatcher/pause merged; GraphQL scheduler/budget enforcement remain |
| 5 | GitHub Issue synchronization | Partial — manual import and linked-Issue updates; automatic Outbox/cursor workers remain |
| 6 | Pull requests, commits, checks, reviews | Partial — core list/detail/actions exist; review-thread polish remains |
| 7 | Inbox, triage, automations, analytics, dashboards | Partial |
| 8 | GitLab/Gitea providers | Fixture-backed/read-only; live sync not implemented |
| 9 | Optional AI extension | Complete as optional/fixture-backed integration |
| 10 | Stabilization and release | Partial — native smoke/signing/export/soak items remain |

## PR #12 — 2026-11-18

Merged as `e380c58098878080b7d746f3cad7889effdc2fb6`. Adds a shared process-wide GitHub REST request gate, mutation pacing, REST request telemetry, primary/secondary REST rate-limit classification and enforced REST pause/resume. CI run [44166546258](https://github.com/mindwork64/forgedesk/actions/runs/44166546258) passed 7/7 jobs.

## P0 remaining

1. Unified REST + GraphQL dispatcher with priorities and shared secondary guard.
2. Enforced 10% background allowance / 20% reserve and adaptive scheduler.
3. Automatic durable Outbox worker with safe reconciliation.
4. Cursor/watermark-driven incremental Issue sync.

Detailed handoff: `docs/specs/P0_REMAINING_SYNC_SPEC.md`.
