# Stage 0 report — Repository reconnaissance & architecture lock

**Date:** 2026-11-15  
**Status:** DONE

## Scope

Start the empty repository, lock the architecture, produce a buildable
Tauri 2 + Rust + Vue 3 + TypeScript skeleton with SQLite-ready core boundary, and
record decisions/assumptions. No product features beyond foundation.

## Repository state found

Empty except the six requirement documents and `README.md`. No existing code, CI
or conventions to preserve. Rust 1.99.1, Node 22.23.2, npm 10.9.8, Git 2.43
available. Tauri system prerequisites (WebKitGTK 4.1) are absent in this sandbox
and sudo is unavailable.

## Delivered

- `Cargo.toml` workspace: `crates/forgedesk-core` (default member) + `src-tauri`.
- Core skeleton with domain, errors, SQLite boundary, migrations, repositories,
  settings and backup/restore foundations (completed in Stage 1).
- `src-tauri`: Tauri 2 builder, typed commands, state, capabilities, window
  persistence, icons.
- Vue/TS/Vite frontend skeleton.
- Architecture: `overview.md`, `repository_map.md`, `assumptions.md`.
- `ADR-0001` stack and core boundary.
- Development setup notes (including read-only HOME workaround).

## Verification

| Command | Result |
|---|---|
| `cargo test -p forgedesk-core` | PASS — 20 tests |
| `cargo clippy -p forgedesk-core --all-targets -- -D warnings` | PASS |
| `cargo fmt --check` | PASS |
| `npm run build` | PASS |
| `npx tauri info` | Config parsed; environment lacks webkit2gtk-4.1 |
| `cargo check -p forgedesk` | BLOCKED locally (missing WebKitGTK dev libs); CI lane required |

## Known limitations

- No CI workflow yet (added in the next iteration alongside Stage 1 checks).
- Tauri shell has not been compiled locally; its code is deliberately thin and
  uses stable Tauri 2 APIs, but CI on Linux/Windows must confirm it.
- No screenshots yet; UI Gate 0 belongs to Stage 1.

## Next

Stage 1: UI Gate 0 shell/design system + complete storage/settings UX.
