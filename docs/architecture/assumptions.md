# Assumptions and chosen defaults

Only decisions that are not fixed by `01`–`06` but are needed to implement them.
Each entry is small and reversible unless noted.

## Storage and lifecycle

1. **Application data directory** comes from Tauri `app_data_dir()`; the database
   is `<data_dir>/forgedesk.db`, managed backups are `<data_dir>/backups/`.
   Attachments will be `<data_dir>/attachments/` when introduced.
2. **One workspace per installation** in v1. The database enforces this in the
   workspace repository inside a transaction. A second onboarding attempt returns
   a `conflict` error, never creates a duplicate.
3. **Settings** are one typed JSON document stored under the `app.settings` key in
   `app_setting(key, value, updated_at)`. Missing or extra JSON fields fall back to
   defaults (`#[serde(default)]`); unreadable JSON is an error, not a silent reset.
4. **Window geometry** (size, position, maximized) is persisted in settings on
   close and restored at startup only when the saved rectangle intersects an
   available monitor.
5. **Auto-backup** runs on startup when the newest managed snapshot is ≥ 24 h old
   and the setting is enabled. Retention defaults to 20 snapshots (bounded
   3–100). A failed automatic backup is a visible startup warning, never a reason
   to block local work.
6. **Restore scope in Stage 1** accepts an opaque managed backup id, not an
   arbitrary file path. This avoids adding a file-picker capability before the
   Integrations/Storage UX exists. Importing an external backup file is deferred.
7. **Restore is a maintenance action**: validate → safety snapshot → close →
   replace → reopen. If reopen fails, the previous file is restored.

## Identity and time

8. Local ids are **UUID v7** strings (time-ordered). Human issue keys `FD-<n>`
   arrive in Stage 2 with a workspace-scoped counter.
9. All timestamps are UTC RFC 3339 with milliseconds; display conversion uses the
   system timezone.

## Frontend

10. **Default theme is Dark** and default locale is `system` with English
    fallback, per `06_UI_UX_SPEC.md` §6.0.
11. **No bundled fonts.** `Inter` is used when installed; fallback is `Segoe UI`
    on Windows and the system sans-serif on Linux, per §3.3.
12. **No UI kit.** Shared primitives are built in-repo on tokens; Lucide is the
    single icon set.
13. **IPC contract** mirrors Rust DTOs with camelCase JSON. All commands go
    through `src/lib/ipc.ts`; components do not import `invoke` directly.
14. **Browser-based screenshots** for UI gates are acceptable evidence when the
    native shell cannot run in the environment (no WebKitGTK dev libraries), as
    long as the report says so and geometry is measured on the same built CSS.

## Security

15. Renderer capability is limited to `core:default`. No fs/shell/http/dialog
    plugin is granted in Stage 1.
16. Logs are structured but minimal in Stage 1 (stderr, level-filtered, no
    payloads or credentials). Log-file rotation and export are Stage 10 work.
17. GitHub credentials will use the OS credential store when the integration
    lands (Stage 4); tokens are never written to SQLite or backups.

## Unresolved, non-blocking

- GitHub App registration (client id, permissions, auth flow choice) is an
  external dependency for Stage 4 and needs user involvement at that point.
- Signing certificates/installer identity are release-time inputs (Stage 10).
- Exact FTS5 tokenizer configuration will be decided in Stage 2 with realistic
  content fixtures.

## Issues and workflow (Stage 2)

18. **Default workflow:** one `Default` workflow per workspace with Backlog, Todo,
    In Progress, Done and Canceled; Backlog is the default create status. Default
    names are localized via `system_key`; user-created names are data.
19. **Human keys:** `FD-<n>` allocated transactionally from the workspace counter;
    the numeric part is stable and never reused.
20. **Filters:** one nested group level in v1, maximum 50 conditions, whitelisted
    fields/operators, limit 500 rows per query (1–1000 API bound). API-level limit
    is a temporary guard until virtualization/pagination lands.
21. **Project references:** Stage 2 includes a minimal project table so issues can
    be assigned/filtered; project creation UI is Stage 3 and is not simulated.
22. **Issue activity:** structured events, not localized strings; the UI localizes
    known kinds and falls back to the raw kind for future events.

23. **Saved views** persist the query (filter/sort/groupBy) plus metadata
    (columns/density/pin/position), not issue ids, so results stay current.
24. **Filter editor scope:** the UI exposes `eq`, `neq`, `contains`, range and
    empty operators; `in`/`notIn` remain backend-only until a multi-value control
    is added.
25. **Virtualization:** flat lists virtualize above 100 rows (40 px rows, 8-row
    overscan); grouped lists are not windowed yet. The API page limit is 500.

## Planning (Stage 3)

26. **Progress:** completed / (total − canceled) for projects, milestones and
    initiatives; initiatives aggregate completed/total across linked projects.
    Archived/deleted issues are excluded; canceled is excluded from the denominator.
27. **Relationships:** a project has at most one initiative; an issue has at most
    one cycle and one milestone in v1 (primary-key join rows). Moving an issue
    replaces the previous link.
28. **Cycle status** is derived from local dates; no stored status and no
    background scheduler.
29. **Documents** are Markdown source with optional project/initiative links;
    export is Markdown text, import/outline are follow-ups.
30. **Roadmap v1** is read-only and plots projects/initiatives; milestones level,
    drag editing and dependencies are deferred.

## Local Git and API budgets (Stage 4)

31. **Git invocation:** direct argv through `Command`, no shell. Repository paths
    are canonicalized and must be attached by the user; branch create/switch require
    a clean worktree and Git's own ref validation.
32. **Credentials:** provider account rows contain identity/scopes/status, never
    tokens. GitHub App Device Flow auth is implemented; tokens live in the OS
    keyring, with session-only memory fallback if no keyring exists. No plaintext
    token file or SQLite token column is allowed.
33. **API budgets:** background allowance is
    `min(10% × limit, max(0, remaining − 20% × limit))`; unknown observations yield
    zero allowance. GitHub-reported remaining is never presented as ForgeDesk usage.
34. **Queue/dispatcher:** queue and request log tables are durable foundation; no
    provider request is simulated or sent until live auth/client exists.
35. **Local repositories:** removing a repository row does not delete files or Git
    data. GitHub discovery is deferred until authentication exists.

## GitHub sync safety (Stage 5)

36. **Remote links:** one remote issue link per local issue; remote IDs/numbers are
    stored separately from local UUIDs. Field ownership is explicit and never
    inferred from timestamps alone.
37. **Outbox:** idempotency keys are unique; ambiguous mutation outcomes enter
    `outcome_unknown` and may only move to `reconciling`, never directly to retry or
    success.
38. **Cursors:** page cursor and time watermark are separate; success timestamps are
    not advanced on partial/failed pages.
39. **No live sync before auth:** Stage 5 foundation is inert until Stage 4 GitHub
    App auth, token vault, provider client and budget-aware dispatcher exist.
