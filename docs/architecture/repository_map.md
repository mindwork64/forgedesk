# Repository map

Updated after Stage 4 (local Git/API-budget foundation). Update only when the structure changes.

```text
ForgeDesk/
├── 01–06_*.md                  Authoritative product/technical/UI requirements
├── README.md                   Project package description
├── CHANGELOG.md                Stage-level change record
├── package.json                Vite/Vue frontend + Tauri CLI + test/visual scripts
├── package-lock.json           Locked npm dependency graph
├── Cargo.toml                  Rust workspace; default member = core crate
├── Cargo.lock                  Locked Rust dependency graph
├── rust-toolchain.toml         Stable toolchain + rustfmt/clippy
├── vite.config.ts              :1420 dev server, build target, Vitest config
├── tsconfig*.json              Strict TypeScript
├── scripts/
│   ├── visual-gate0.mjs        Gate 0/1 screenshots + geometry assertions
│   ├── ci.sh                   Shared local/Jenkins verification pipeline
│   ├── ci-sync.sh              Publish tree to the Jenkins mirror
│   ├── ci-serve.sh             Read-only git daemon for Jenkins
│   ├── jenkins-create-job.sh   Create/update ForgeDesk-CI
│   └── jenkins-trigger.sh      Queue a Jenkins build
├── Jenkinsfile                 ForgeDesk-CI declarative pipeline
├── ci/
│   ├── pkgconfig/              Typecheck-only WebKitGTK shims
│   └── jenkins/ForgeDesk-CI.xml Versioned job definition
├── .github/workflows/ci.yml    Frontend, core, shell and visual-gate lanes
│
├── crates/forgedesk-ai/src/
│   ├── lib.rs                  Optional AI providers/context/approval service
│   └── bin/forgedesk-mcp.rs    Read-only MCP STDIO bridge
│
├── crates/forgedesk-core/src/
│   ├── lib.rs                  Public core surface
│   ├── app.rs                  AppService: bootstrap, settings, backups, issue operations
│   ├── error.rs                CoreError + stable WireError categories
│   ├── backup.rs               VACUUM INTO snapshots, manifests, validation, prune
│   ├── git.rs                  Safe direct-argv Git process wrapper
│   ├── github_auth.rs          GitHub App Device Flow + keyring token vault
│   ├── github_dispatcher.rs    Shared REST single-flight gate, mutation pacing, telemetry and pause enforcement (GraphQL/priority scheduler pending)
│   ├── provider.rs             GitLab/Gitea adapter contract + transports
│   ├── github_client.rs        GitHub REST client; production calls use shared dispatcher
│   ├── github_pr.rs            PR detail/commits/reviews/check-runs
│   ├── github_activity.rs      Review queue + CI/check-runs reads
│   ├── github_actions.rs       Permissioned merge/close/review actions
│   ├── domain/inbox.rs         Inbox event/filter DTOs
│   ├── domain/automation.rs    Automation rule/run DTOs
│   ├── domain/dashboard.rs     Dashboard/widget DTOs
│   ├── domain/
│   │   ├── api_budget.rs       Budget observation/request/queue DTOs + formula
│   │   ├── git.rs              Repository/Git status/branch/log/diff DTOs
│   │   ├── issue.rs            Issue/status/label/filter/query DTOs + validation
│   │   ├── planning.rs         Project/initiative/milestone/cycle/document DTOs
│   │   ├── sync.rs             Remote link/outbox/cursor/conflict DTOs
│   │   ├── saved_view.rs       Saved view DTO/validation
│   │   ├── settings.rs         Typed AppSettings, patch validation, window state
│   │   ├── workspace.rs        Single workspace validation
│   │   └── time.rs             UTC RFC 3339 helpers
│   └── db/
│       ├── mod.rs              Guarded SQLite connection, WAL, reopen/replace
│       ├── migrations.rs       Append-only runner + tests
│       ├── migrations/0001_init.sql … 0011_project_issue_key_prefix.sql
│       └── repositories/
│           ├── issues.rs       Issue CRUD/detail/list/batch/duplicate
│           ├── issue_query.rs  Whitelisted filter/sort/search SQL builder
│           ├── workflow.rs     Default workflow/status seed/load
│           ├── labels.rs       Labels + issue-label links
│           ├── relations.rs    Blocks/related/duplicate relations
│           ├── comments.rs     Local comments
│           ├── activity.rs     Structured activity log
│           ├── projects.rs     Minimal issue project refs
│           ├── planning.rs     Projects/initiatives/milestones/cycles/documents
│           ├── repo_registry.rs Local repository registry
│           ├── api_budget.rs   Observations, request log, queue, control
│           ├── saved_views.rs  Persisted filter/sort/group views
│           ├── sync.rs         Sync links/cursors/outbox/conflicts
│           ├── pull_links.rs   PR↔local issue links
│           ├── inbox.rs        Inbox normalization/read state
│           ├── automation.rs   Local automation rules/preview/runs
│           ├── dashboard.rs    Persisted dashboard layout/widgets
│           ├── settings.rs     Settings JSON document
│           └── workspace.rs    Workspace + human-key counter
│
├── src-tauri/
│   ├── Cargo.toml              Tauri 2 shell crate
│   ├── build.rs
│   ├── tauri.conf.json         Window geometry, CSP, bundle, icons
│   ├── capabilities/default.json  Minimal `core:default` capability
│   ├── icons/                  Generated Windows/Linux/macOS icon set
│   └── src/
│       ├── main.rs             Entry point
│       ├── lib.rs              Builder, setup, window persistence, shutdown
│       ├── state.rs            AppState, startup-error retention
│       └── commands/           Typed IPC commands + CommandError
│
├── src/
│   ├── main.ts                 Pinia/router/i18n setup, dev harness gate
│   ├── App.vue                 Boot/loading, startup error, shell, onboarding
│   ├── navigation.ts           Fixed nav groups/order, pinned-view model
│   ├── router/index.ts         Hash routes and page metadata
│   ├── types/ipc.ts            IPC DTO mirror types
│   ├── lib/
│   │   ├── ipc.ts              Typed command facade + error normalization
│   │   ├── routes.ts           Internal route validation
│   │   ├── theme.ts            Theme paint hint
│   │   ├── format.ts           Locale-aware date/byte formatting
│   │   └── clipboard.ts        Secure-context + fallback copy
│   ├── stores/
│   │   ├── app.ts              Bootstrap, serialized settings writes, backups
│   │   ├── issues.ts           Issue list/detail/mutations, request guards
│   │   ├── planning.ts         Projects/initiatives/cycles/documents state
│   │   ├── git.ts              Repositories/Git overview actions
│   │   ├── githubAuth.ts       Device Flow/connection/repo selection
│   │   ├── pullRequests.ts     PR list/detail state
│   │   ├── inbox.ts            Inbox filters/read/archive state
│   │   ├── automations.ts      Rule toggle/preview/run/history
│   │   ├── dashboard.ts        Layout/widget mutations
│   │   ├── apiMonitor.ts       API Monitor state/pause/diagnostics
│   │   ├── ui.ts               Overlays, groups, viewport, chord hint
│   │   └── toasts.ts           Bounded toast queue
│   ├── i18n/                   en/ru catalogues + locale/theme resolution
│   ├── styles/
│   │   ├── tokens.css          Design-token source of truth
│   │   ├── base.css            Reset, typography, focus, scrollbars, tooltip
│   │   └── tokens.spec.ts      Contract test for spec values
│   ├── directives/tooltip.ts   Fixed-position tooltip overlay
│   ├── composables/useShortcuts.ts  Global Ctrl+K/chords/help
│   ├── components/
│   │   ├── shell/              AppShell, sidebar, header, breadcrumbs, status
│   │   ├── layout/             PageHeader, PageToolbar
│   │   ├── issues/             List/row/board/detail/create + filter/sort/views
│   │   ├── planning/           Progress bar and planning status badge
│   │   ├── ui/                 Fd* shared primitives
│   │   ├── CommandPalette.vue
│   │   ├── ShortcutsDialog.vue
│   │   ├── OnboardingFlow.vue
│   │   └── StartupError.vue
│   ├── features/
│   │   ├── issues/MyIssuesView.vue
│   │   ├── projects/           Projects list/overview/dialogs
│   │   ├── initiatives/        Initiatives list/overview/dialogs
│   │   ├── cycles/             Cycles view/dialog
│   │   ├── documents/          Documents list + Markdown editor
│   │   ├── roadmap/            Projects/initiatives timeline
│   │   ├── repositories/       Local repository list/detail/dialog
│   │   ├── integrations/       Provider cards + GitHub Device Flow dialog
│   │   ├── pull-requests/      PR table + read-only detail pane
│   │   ├── reviews/            Requested-review queue
│   │   ├── ci/                 Commits + check-runs
│   │   ├── inbox/              Inbox event list/triage
│   │   ├── automations/        Local rules, preview, history
│   │   ├── dashboards/         Persisted widget canvas/edit mode
│   │   ├── ai/                 Optional AI settings/context/suggestion UI
│   │   ├── api-monitor/        Budgets, queue, request history
│   │   ├── placeholder/PlaceholderView.vue
│   │   ├── settings/           Nine settings sections + rows/sections
│   │   └── about/AboutView.vue
│   └── dev/
│       ├── visualHarness.ts    Build-gated visual fixture (not production)
│       ├── issueFixtures.ts    Deterministic issue/workflow fixtures for gates
│       ├── planningFixtures.ts Stage 3 planning/document fixtures for gates
│       └── stage4Fixtures.ts   Git/API budget fixtures for gates
│
├── docs/
│   ├── PROJECT_STATE.md        Compact current state
│   ├── architecture/           overview, repository_map, assumptions
│   ├── decisions/              ADR-0001…0010 (…/provider-adapters/optional-AI)
│   ├── development/            setup.md, jenkins.md
│   ├── stages/                 Stage 0/1/2/3/4/5/6/7/8/9/10 + repository report
│   ├── release/                Release checklist and blockers
│   ├── ui/
│   │   ├── component_map.md
│   │   ├── visual_baseline.md
│   │   ├── UI-GATE-0-REPORT.md
│   │   ├── UI-GATE-1-REPORT.md
│   │   ├── UI-GATE-2-REPORT.md
│   │   ├── UI-GATE-3-REPORT.md
│   │   └── current/            Gate 0/1 screenshots + measurements
│   └── notes/                  (reserved)
└── tests/                      (later: integration, fixtures, provider contracts)
```

## Dependency direction

`src` → typed IPC → `src-tauri/commands` → `forgedesk-core::app` → `domain` + `db`.
`db` never imports `src-tauri`; `domain` never imports `db`, Tauri, HTTP or provider
adapters. Provider/sync/API-budget modules will depend on domain DTOs, not the
reverse.
