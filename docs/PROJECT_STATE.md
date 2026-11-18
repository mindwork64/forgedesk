# PROJECT_STATE

**Snapshot:** 2026-11-18  
**Main:** e380c58098878080b7d746f3cad7889effdc2fb6  
**SQLite schema:** v11

## Recent merged work

- PR #8: Project Archive/Restore, issue-key prefixes, Search improvements.
- PR #10: manual bidirectional sync for linked GitHub Issues, Outbox/reconciliation and conflict resolution.
- PR #11: GitHub status messaging and PR detail pane lifecycle fix.
- PR #12: shared single-flight GitHub REST dispatcher, mutation pacing, REST telemetry, primary/secondary REST rate classification and enforced API Monitor Pause/Resume.
- PR #12 CI: 44166546258, 7/7 jobs passed at tested head 405f3d4cf4080a49f154d7d29eea53bade4857b4.

## Implemented areas

Local shell/Core/SQLite, backup/restore, RU/EN, themes; Issues CRUD/workflow/List/Board/filters/Saved Views; Projects/Initiatives/Milestones/Cycles/Documents/Roadmap; safe Local Git subset; GitHub auth/repository discovery/Issue import/PR/review/check/actions; manual linked-Issue sync; Inbox/basic Automations/Insights/Dashboards; fixture-based GitLab/Gitea read adapters; optional AI extension.

## Full-product gaps

- Issues: templates/watchers/attachments/milestone-cycle UI/workflow editor/Saved View columns-density/in-notIn/comment edit-delete/Table-Calendar-Timeline.
- Planning/Documents: project templates/progress modes/cycle board and analytics/roadmap milestone drag-dependencies/document revisions-import-outline-search-attachments/quick-create context.
- Local Git: timeout/cancel/open folder-terminal-IDE/branch delete-rebase/guarded remote operations.
- GitHub: GraphQL shared dispatch, budget scheduler, ETag cache, Issue create/comments/milestones, automatic Outbox/reconciliation/cursors, full PR review-thread and API logging.
- Productivity: triage, full automation, complete metrics/widgets.
- GitLab/Gitea: live account lifecycle and supported operations beyond mock read-only adapters.
- AI: remaining generators/streaming/model usage; optional/default-off.
- Release: native interactive Windows/Linux, export/import attachments, performance/soak, packaging/signing/user docs.

## Current objective

Complete all ForgeDesk functionality from 01–06, not only P0. Master backlog/acceptance/DeepSeek starter prompt: docs/specs/PRODUCT_COMPLETION_SPEC.md. Detailed GitHub sync sub-spec: docs/specs/P0_REMAINING_SYNC_SPEC.md.

## Locked architecture

Single-user local-first desktop: Rust Stable Core + Tauri 2 + Vue 3/TypeScript + SQLite. No backend, PostgreSQL, Redis, mandatory AI or telemetry. Windows/Linux. GitHub primary; capability-based GitLab/Gitea optional. AI disabled by default.

## Verification boundary

PR #12 CI is green 7/7. CI/browser screenshots are not native interactive runtime tests; provider fixtures are not live-host tests. Signed release requires external signing credentials.
