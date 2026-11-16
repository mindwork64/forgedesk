# UI Gate 4 — Inbox/triage

**Date:** 2026-11-15
**Scope:** Stage 7 Inbox first slice.
**Status:** PARTIAL — Inbox accepted; Automations/Insights/Dashboards not yet built.

## Capture

| # | File | Size | Content |
|---|---|---|---|
| 1 | `docs/ui/current/gate4-inbox-dark-1440.png` | 1440×900 | Inbox groups, filters, unread markers |
| 2 | `docs/ui/current/gate4-insights-dark-1440.png` | 1440×900 | Insights metrics/charts |
| 3 | `docs/ui/current/gate4-automations-dark-1440.png` | 1440×900 | automation rules and actions |
| 4 | `docs/ui/current/gate4-dashboards-dark-1440.png` | 1440×900 | persisted dashboard widgets |

Full run: **59 captures** (17 Gate 0 + 13 Gate 1 + 14 Gate 2 + 15 Gate 3 + 4 Gate 4).

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
| Insights local aggregates | PASS | summary, throughput, status/project distribution |
| Automations local rules/preview/run | PASS | toggle, preview, run, history |
| Dashboards layout/widgets | PASS | persisted 12-column edit mode |
| GitLab/Gitea capability cards | PASS | read-only adapter ready, credentials not configured |

## Accessibility

- axe-core `button-name`/`aria-command-name` runs for all 59 visual cases.
- Regression fixtures prove valid names pass and empty/hidden-only/role-button failures
  are rejected.

## Notes

- Fixture events are deterministic and only used behind `VITE_VISUAL_HARNESS`.
- Detail pane opens on row click; current build has no dedicated issue deep-link yet.
