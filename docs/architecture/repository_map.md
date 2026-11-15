# Repository map

Updated after Stage 1. Update only when the structure changes.

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
├── scripts/visual-gate0.mjs    UI Gate 0 screenshot + geometry assertions
├── .github/workflows/ci.yml    Frontend, core, shell and visual-gate lanes
│
├── crates/forgedesk-core/src/
│   ├── lib.rs                  Public core surface
│   ├── app.rs                  AppService: bootstrap/workspace/settings/backup/restore
│   ├── error.rs                CoreError + stable WireError categories
│   ├── backup.rs               VACUUM INTO snapshots, manifests, validation, prune
│   ├── domain/
│   │   ├── settings.rs         Typed AppSettings, patch validation, window state
│   │   ├── workspace.rs        Single workspace validation
│   │   └── time.rs             UTC RFC 3339 helpers
│   └── db/
│       ├── mod.rs              Guarded SQLite connection, WAL, reopen/replace
│       ├── migrations.rs       Append-only runner + tests
│       ├── migrations/0001_init.sql
│       └── repositories/       settings + workspace SQL
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
│   │   ├── ui/                 Fd* shared primitives
│   │   ├── CommandPalette.vue
│   │   ├── ShortcutsDialog.vue
│   │   ├── OnboardingFlow.vue
│   │   └── StartupError.vue
│   ├── features/
│   │   ├── issues/MyIssuesView.vue
│   │   ├── placeholder/PlaceholderView.vue
│   │   ├── settings/           Nine settings sections + rows/sections
│   │   └── about/AboutView.vue
│   └── dev/visualHarness.ts    Build-gated visual fixture (not production)
│
├── docs/
│   ├── PROJECT_STATE.md        Compact current state
│   ├── architecture/           overview, repository_map, assumptions
│   ├── decisions/              ADR-0001 stack/core, ADR-0002 UI foundation
│   ├── development/setup.md    Toolchain, commands, sandbox notes
│   ├── stages/                 Stage 0/1 reports
│   ├── ui/
│   │   ├── component_map.md
│   │   ├── visual_baseline.md
│   │   ├── UI-GATE-0-REPORT.md
│   │   └── current/            Gate 0 screenshots + measurements
│   └── notes/                  (reserved)
└── tests/                      (later: integration, fixtures, provider contracts)
```

## Dependency direction

`src` → typed IPC → `src-tauri/commands` → `forgedesk-core::app` → `domain` + `db`.
`db` never imports `src-tauri`; `domain` never imports `db`, Tauri, HTTP or provider
adapters. Provider/sync/API-budget modules will depend on domain DTOs, not the
reverse.
