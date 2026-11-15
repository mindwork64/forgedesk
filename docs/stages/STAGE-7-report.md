# Stage 7 report — Inbox/triage (partial)

**Date:** 2026-11-15
**Status:** PARTIAL — Inbox over normalized local activity and local Insights are
delivered; external event ingestion, automations and Dashboards remain open.
**Schema:** v8 (`inbox_state`, `automation_rule`, `automation_run`)

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
- **Insights:** local summaries (created/completed/open/overdue, average cycle),
  12-bucket throughput, status/project distributions and `data through`; range
  30/90/365 days. No external data is presented as live.

## Not delivered

- GitHub event ingestion/dedup into Inbox (requires sync event pipeline).
- Dashboards.
- Triage routing/assignment actions.
- Automations: rule model/editor, preview, run history, loop/dedupe protection.
- Insights aggregates/charts and Dashboards.
- Native Tauri verification.

## Verification

| Check | Command | Result |
|---|---|---|
| Core tests | `cargo test -p forgedesk-core` | PASS — 83 (+3 ignored live) |
| Frontend | `npm run typecheck`, `npm test` | PASS — 107 |
| Visual gates | `node scripts/visual-gate0.mjs` | PASS — 56 captures |
| Inbox capture | `gate4-inbox-dark-1440.png` | PASS — grouped rows, filters, unread markers |

## Next

1. Normalize GitHub/PR/sync events with dedupe keys.
2. Triage actions and assignment routing.
3. Dashboards with persisted 12-column layout.
4. External event ingestion for Inbox.
