# Full ForgeDesk product roadmap

This page summarizes docs/specs/PRODUCT_COMPLETION_SPEC.md, which is the normative master handoff.

## Local product

- Issues: templates, watchers, managed attachments, milestone/cycle fields and complete comments/filters/views.
- Workflow editor and functional Saved View columns/density; List, Board, Table, Calendar, Timeline.
- Planning: project templates/progress methods, cycle board/velocity/burnup/burndown, roadmap milestone level/dependencies.
- Documents: templates, revision history, Markdown import/export, outline, in-document search and attachments.
- Local Git: timeout/cancel, open folder/terminal/IDE, branch delete/rebase and confirmed remote actions.

## External workflows

- Shared REST+GraphQL dispatcher, priority/fairness, ETag cache, budget enforcement, adaptive scheduler and API Monitor.
- GitHub Issue create/update/comments/labels/milestones with Outbox, reconciliation, ownership/conflicts and incremental cursor/watermark sync.
- PR inline comments/thread resolution, GraphQL summaries, CI/review budget accounting and tags/releases.
- Inbox/Triage, full event-driven automations, metrics and dashboard widgets.
- GitLab/Gitea secure live account flows and capability-based operations.
- Optional AI plan/subtask/acceptance/PR generators, context preview, structured validation, streaming/usage; default off.

## Release closure

- Windows/Linux native interactive smoke tests.
- Workspace export/import including managed attachments.
- Schema v11 onward data-preserving migrations.
- Large-workspace performance and long-running sync soak.
- Accessibility/keyboard, Dark/Light and RU/EN.
- Signed installers when credentials are available; otherwise clearly labeled unsigned artifacts.
- User installation/update/uninstall, backup/restore and troubleshooting docs.

## Exclusions

No multi-user collaboration, backend/cloud DB, Redis, webhook relay, mandatory AI/telemetry, auto-push/auto-merge or full provider mirror.

The app is complete only when mandatory 01–06 requirements have traceability and PASS evidence, native Windows/Linux flows are verified and release blockers are explicit.
