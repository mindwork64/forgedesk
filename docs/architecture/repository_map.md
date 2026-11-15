# Repository map

Updated after Stage 3 (planning vertical slice). Update only when the structure changes.

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
├── crates/forgedesk-core/src/
│   ├── lib.rs                  Public core surface
│   ├── app.rs                  AppService: bootstrap, settings, backups, issue operations
│   ├── error.rs                CoreError + stable WireError categories
│   ├── backup.rs               VACUUM INTO snapshots, manifests, validation, prune
│   ├── domain/
│   │   ├── issue.rs            Issue/status/label/filter/query DTOs + validation
│   │   ├── planning.rs         Project/initiative/milestone/cycle/document DTOs
│   │   ├── saved_view.rs       Saved view DTO/validation
│   │   ├── settings.rs         Typed AppSettings, patch validation, window state
│   │   ├── workspace.rs        Single workspace validation
│   │   └── time.rs             UTC RFC 3339 helpers
│   └── db/
│       ├── mod.rs              Guarded SQLite connection, WAL, reopen/replace
│       ├── migrations.rs       Append-only runner + tests
│       ├── migrations/0001_init.sql, 0002_issues.sql, 0003_planning.sql
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
│           ├── saved_views.rs  Persisted filter/sort/group views
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
│   │   ├── placeholder/PlaceholderView.vue
│   │   ├── settings/           Nine settings sections + rows/sections
│   │   └── about/AboutView.vue
│   └── dev/
│       ├── visualHarness.ts    Build-gated visual fixture (not production)
│       ├── issueFixtures.ts    Deterministic issue/workflow fixtures for gates
│       └── planningFixtures.ts Stage 3 planning/document fixtures for gates
│
├── docs/
│   ├── PROJECT_STATE.md        Compact current state
│   ├── architecture/           overview, repository_map, assumptions
│   ├── decisions/              ADR-0001 stack, 0002 UI, 0003 issues, 0004 planning
│   ├── development/            setup.md, jenkins.md
│   ├── stages/                 Stage 0/1/2/3 reports
│   ├── ui/
│   │   ├── component_map.md
│   │   ├── visual_baseline.md
│   │   ├── UI-GATE-0-REPORT.md
│   │   ├── UI-GATE-1-REPORT.md
│   │   ├── UI-GATE-2-REPORT.md
│   │   └── current/            Gate 0/1 screenshots + measurements
│   └── notes/                  (reserved)
└── tests/                      (later: integration, fixtures, provider contracts)
```

## Dependency direction

`src` → typed IPC → `src-tauri/commands` → `forgedesk-core::app` → `domain` + `db`.
`db` never imports `src-tauri`; `domain` never imports `db`, Tauri, HTTP or provider
adapters. Provider/sync/API-budget modules will depend on domain DTOs, not the
reverse.
