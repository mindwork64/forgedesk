# UI Gate 4 — Inbox/triage

**Date:** 2026-11-15
**Scope:** Stage 7 Inbox first slice.
**Status:** PARTIAL — Inbox accepted; Automations/Insights/Dashboards not yet built.

## Capture

| # | File | Size | Content |
|---|---|---|---|
| 1 | `docs/ui/current/gate4-inbox-dark-1440.png` | 1440×900 | Inbox groups, filters, unread markers |

Full run: **54 captures** (16 Gate 0 + 13 Gate 1 + 9 Gate 2 + 15 Gate 3 + 1 Gate 4).

## Acceptance

| Criterion | Result | Evidence |
|---|---|---|
| Compact vertical event list, no card tiles | PASS | `.fd-inbox-row` |
| All/Unread/GitHub/Local/Sync filters | PASS | segmented tabs |
| Today/Yesterday/Earlier grouping | PASS | three groups in fixture |
| Unread marker and source badge | PASS | dots + source labels |
| Mark all read / row actions | PASS | header + icon actions |
| Empty/loading/error states | PASS | components wired |
| External sync events | NOT IMPLEMENTED | no producer yet |
| Automations/Analytics/Dashboards | NOT IMPLEMENTED | next slices |

## Notes

- Fixture events are deterministic and only used behind `VITE_VISUAL_HARNESS`.
- Detail pane opens on row click; current build has no dedicated issue deep-link yet.
