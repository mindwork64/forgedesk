# Development

## Requirements

- Rust stable (see `rust-toolchain.toml`), Node.js 22+, npm.
- Linux desktop build needs WebKitGTK 4.1 / libsoup 3; Windows needs WebView2.
- Git CLI is required for local repository features.

## Commands

```bash
npm ci
npm run tauri dev        # native desktop
npm run dev              # renderer only
npm run typecheck
npm test
npm run build
cargo test -p forgedesk-core
cargo clippy -p forgedesk-core --all-targets -- -D warnings
./scripts/ci.sh          # full local gate incl. screenshots
```

The visual harness is build-gated by `VITE_VISUAL_HARNESS=true`; production builds
must not contain it. `scripts/ci.sh` runs UI Gate 0–3 screenshots with Playwright and
then rebuilds production to verify harness absence.

## Local Jenkins

`docs/development/jenkins.md` documents the local mirror/git-daemon workflow.
Credentials are read only from `JENKINS_URL`, `JENKINS_USER`, `JENKINS_TOKEN`
environment variables and are never committed.

## Repository layout

See `docs/architecture/repository_map.md`.
