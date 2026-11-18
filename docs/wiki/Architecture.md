# Architecture

## Stack and boundary

- Rust Stable Core, Tauri 2 shell, Vue 3 + TypeScript renderer and SQLite.
- Local-first, single-user desktop; local workflows work offline; no required backend.
- UI → typed IPC → Tauri commands/AppState → AppService → domain/repositories → SQLite/provider clients.
- Renderer has no arbitrary filesystem/network/SQL/shell access.
- Credentials stay in OS credential store; logs and exports exclude tokens.

## GitHub API runtime

PR #12 merged 2026-11-18 as e380c58098878080b7d746f3cad7889effdc2fb6. Production GitHubClient REST calls use a process-wide single-flight gate, mutation pacing, REST response telemetry, rate-limit classification and persisted pause enforcement. CI run 44166546258 passed 7/7.

This implementation is REST-only. GraphQL transport through the same dispatcher, priority scheduling, enforced background budgets/adaptive scheduler and automatic incremental sync remain unfinished.

## Sync persistence

Existing schema contains remote links, sync_outbox, sync_cursor, sync_conflict, api_budget_observation, api_request_log, api_queue_job and api_control. These are persistence foundations; their existence is not proof of active workers. Current Issue import and linked Issue Outbox dispatch remain manually initiated.

## Full completion scope

Remaining tracks include local Issue/Workflow/View completion; Planning/Documents; safe Local Git; full REST+GraphQL budget/scheduler; GitHub Issue/comment/cursor sync; PR review threads; Inbox/Triage/Automation/Insights; live GitLab/Gitea capabilities; optional AI; native Windows/Linux and release hardening.

See docs/specs/PRODUCT_COMPLETION_SPEC.md and docs/PRODUCT_GAPS_AUDIT.md.
