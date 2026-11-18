# Development

## Requirements

- Rust stable (see `rust-toolchain.toml`), Node.js 22+, npm.
- Linux desktop build needs WebKitGTK 4.1 / libsoup 3 and libdbus-1-dev (OS keyring).
- Windows needs WebView2.
- Git CLI is required for local repository features.

## Commands

~~~bash
npm ci
npm run tauri dev        # native desktop
npm run dev              # renderer only
npm run typecheck
npm test
npm run build
cargo test -p forgedesk-core
cargo clippy -p forgedesk-core --all-targets -- -D warnings
./scripts/ci.sh          # full local gate incl. screenshots
~~~

The visual harness is build-gated by `VITE_VISUAL_HARNESS=true`; production builds
must not contain it. `scripts/ci.sh` runs UI Gate 0–3 and verifies harness absence
from the production build.

## GitHub sync development

- Current verified baseline: PR #12 merge `e380c58098878080b7d746f3cad7889effdc2fb6`.
- PR #12 CI: [run 44166546258](https://github.com/mindwork64/forgedesk/actions/runs/44166546258), 7/7 jobs.
- Remaining work is staged in
  [P0_REMAINDER_DEEPSEEK_TZ.md](../implementation/P0_REMAINDER_DEEPSEEK_TZ.md).
- Read `docs/PROJECT_STATE.md`, `03_GITHUB_API_BUDGET_AND_SYNC.md` and
  `04_TOKEN_EFFICIENCY_RULES.md` before implementation.
- Run targeted tests while iterating; run the full gate on stage/PR boundaries.
- Never claim a native Windows/Linux smoke test from a compile-only CI job.

## Local Jenkins

`docs/development/jenkins.md` documents the local mirror/git-daemon workflow.
Credentials are read only from environment variables and are never committed.

## Repository layout

See `docs/architecture/repository_map.md`.
