# API Budget and Sync — P0 subsystem only

> This wiki page covers the GitHub API/Sync subsystem, not the whole ForgeDesk completion plan. The master handoff for every subsystem and all release work is docs/specs/PRODUCT_COMPLETION_SPEC.md. See the Product-Roadmap wiki page for the complete project scope.

# API Budget and Sync

## Delivered baseline — PR #12

Merged 2026-11-18 as `e380c58098878080b7d746f3cad7889effdc2fb6`. Production `GitHubClient` REST requests use a process-wide single-flight gate; REST mutations are spaced by at least one second; REST telemetry and primary/secondary rate-limit observations are persisted; API Monitor pause is enforced before authenticated REST calls. Pause/Resume is localized EN/RU. CI [44166546258](https://github.com/mindwork64/forgedesk/actions/runs/44166546258) passed 7/7 jobs.

**Boundary:** PR #12 is REST-only and does not provide GraphQL dispatch or automatic background sync. OAuth Device Flow/token exchange/refresh are separate auth operations; authenticated `GET /user` uses the REST client.

## Remaining P0

1. **Unified dispatcher:** REST + GraphQL use one principal-aware single-flight queue, shared secondary guard, real priorities/fairness and cross-API mutation pacing. GraphQL errors must be handled even for HTTP 200.
2. **Budget enforcement:** independent REST resource and GraphQL points buckets. Enforce `min(10% of observed limit, max(0, observed remaining − 20% reserve))` for background work. Unknown/stale budget blocks background.
3. **Adaptive scheduler:** local worker starts after auth, uses persisted due times/jitter/backoff, stops for pause/offline/disconnect/app exit and recovers on restart.
4. **Outbox worker:** automatically dispatch supported linked-Issue `update_issue` jobs; durable claim, definite-vs-ambiguous classification, reconciliation before retry, no blind replay.
5. **Incremental sync:** per-account/repo/resource/query scope, opaque page cursor plus updated_at watermark/overlap, atomic page+checkpoint commit, resumable after restart.

## Persistence

- `sync_cursor` — durable page/watermark checkpoint and schedule.
- `sync_outbox` — durable outbound mutations and idempotency/reconciliation.
- `sync_conflict` — explicit conflict data.
- `api_queue_job` — observable API queue execution state, not a raw payload store.
- `api_budget_observation` / `api_request_log` — sanitized provider budget/request telemetry.

Existing tables are foundations, not evidence of active workers. Schema v11 is current; all changes require append-only migrations.

## Safety

- Never retry an ambiguous mutation without reconciliation.
- Do not treat every 403 as rate limit or every 404 as confirmed deletion.
- No network I/O while holding global app locks or DB write transactions.
- Pause blocks new REST/GraphQL API requests; Resume does not bypass provider reset/secondary pause.
- No server, cron, Redis, cloud queue or webhook relay is required.

Detailed scope and tests: `docs/specs/P0_REMAINING_SYNC_SPEC.md`.
