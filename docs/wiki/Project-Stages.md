# Project stages

| Stage | Scope | Status |
|---|---|---|
| 1 | Desktop foundation, SQLite, settings, backups, UI shell | Complete |
| 2 | Issues, workflow, search, saved views, board | Complete |
| 3 | Projects, initiatives, milestones, cycles, documents, roadmap | Partial |
| 4 | Local Git, repositories, API budget foundation, API Monitor | Partial |
| 5 | GitHub Issue synchronization | Auth done; live sync/client pending |
| 6 | Pull requests, commits, checks, reviews | Not started |
| 7 | Inbox, triage, automations, analytics, dashboards | Not started |
| 8 | GitLab/Gitea providers | Not started |
| 9 | Optional AI extension | Not started |
| 10 | Stabilization and release | Not started |

## Stage 3 follow-ups

Templates, document import/outline, cycle board/analytics and roadmap
milestones/drag/dependency overlays.

## Stage 4 follow-ups

GitHub App Device Flow, secure token vault, live REST/GraphQL client, rate-limit
header parsing, serialized dispatcher, repository discovery and live API Monitor
observations.

## Stage 5 status

The offline safety foundation exists (remote links, cursors, idempotent outbox state
machine, conflicts) and GitHub App Device Flow auth/token refresh is implemented and
live-tested. Live import/export and sync UI are still pending the provider client,
dispatcher and conflict screen. No mass sync or write is enabled yet.
