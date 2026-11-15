# Project stages

| Stage | Scope | Status |
|---|---|---|
| 1 | Desktop foundation, SQLite, settings, backups, UI shell | Complete |
| 2 | Issues, workflow, search, saved views, board | Complete |
| 3 | Projects, initiatives, milestones, cycles, documents, roadmap | Partial |
| 4 | Local Git, repositories, API budget foundation, API Monitor | Partial |
| 5 | GitHub Issue synchronization | Read-only import done; writes/conflicts pending |
| 6 | Pull requests, commits, checks, reviews | Read-only list/detail/files done; actions pending |
| 7 | Inbox, triage, automations, analytics, dashboards | Not started |
| 8 | GitLab/Gitea providers | Not started |
| 9 | Optional AI extension | Not started |
| 10 | Stabilization and release | Not started |

## Stage 3 follow-ups

Templates, document import/outline, cycle board/analytics and roadmap
milestones/drag/dependency overlays.

## Stage 4 follow-ups

Serialized dispatcher, GraphQL client, budget request logs and live API Monitor
observations. Device Flow, token vault, REST reads and repository discovery are
implemented.

## Stage 5 status

The safety model, GitHub App auth, repository discovery and read-only issue import
are implemented and live-tested. Comment import, local→remote writes, conflict UI
and the outbox dispatcher remain open. No GitHub write is enabled yet.

## Stage 6 status

Read-only pull request list/detail with commits, checks, reviews and unified file
patches is implemented and live-tested. Split diff/line comments, review submission,
merge/close, review queue and CI screen remain open.
