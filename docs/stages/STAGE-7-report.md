# Stage 7 report — Inbox/triage/automation/analytics

**Date:** 2026-11-15
**Status:** COMPLETE (DoD) — Inbox, Insights, Automations, Dashboards, external PR
event ingestion and merge→task completion are delivered and live-tested. Real-time
webhooks and assignee routing remain out of v1 scope.
**Schema:** v9 (`inbox_state`, `automation_rule/run`, `dashboard`, `dashboard_widget`)

## Delivered

- `inbox_state` read/archive state keyed to `activity_event`.
- Inbox normalizes local issue activity kinds (created, updated, status/labels,
  comments, archive, relations, duplicate) into title/summary/source/entity.
- All / Unread / GitHub / Local / Sync filters; sync currently has no producer.
- Today / Yesterday / Earlier groups, unread markers, relative time, source badges.
- Mark read/unread, archive, mark all read; row click opens detail pane and marks read.
- Empty/loading/error states; no fake external events.
- Schema v7/v8 migrations and repository tests.
- **Automations v1:** local `issue_created`/`status_changed` triggers, add-label
  actions, enabled toggle, preview, idempotent per-event runs, run history and last
  result. No remote writes and no loop/duplicate execution.
- **External PR events:** linked PR state changes are ingested on PR refresh with
  dedupe and become GitHub-source Inbox events (`pr_merged`, `pr_state_changed`).
- **End-to-end completion:** merging a linked PR marks the local issue Done, records
  activity and updates link state. Live test: test repo PR merged → local issue Done.
- **Dashboards:** schema v9 `dashboard`/`dashboard_widget`, persisted 12-column layout,
  default metric/recent-issues/API-budget widgets, edit mode with add/remove/width
  and reorder. Data is local; no live external claims.
- **Insights:** local summaries (created/completed/open/overdue, average cycle),
  12-bucket throughput, status/project distributions and `data through`; range
  30/90/365 days. No external data is presented as live.

## Not delivered

- Real-time GitHub webhooks (current ingestion is on-demand PR refresh).
- Assignee/triage routing (no assignee field in v1 schema).
- Native Tauri verification.

## Verification

| Check | Command | Result |
|---|---|---|
| Core tests | `cargo test -p forgedesk-core` | PASS — 87 (+3 ignored live) |
| Frontend | `npm run typecheck`, `npm test` | PASS — 107 |
| Visual gates | `node scripts/visual-gate0.mjs` | PASS — 57 captures |
| Inbox capture | `gate4-inbox-dark-1440.png` | PASS — grouped rows, filters, unread markers |
| Live merge→completion | test repo PR #5 | PASS — issue Done, `pr_merged` activity |
| Live PR event sync | test repo PR #6 | PASS — closed event ingested once, dedupe on repeat |

## Next

1. Stage 8 provider adapters and capability contracts.
2. Real-time webhooks and assignee/triage routing.
3. Native Tauri verification.
