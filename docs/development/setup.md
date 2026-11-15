# Local development setup

## Requirements

- Rust stable (workspace `rust-version = 1.85`; verified with 1.99; toolchain file
  pins `stable`)
- Node.js 22+ and npm 10+
- Git
- Linux (Tauri): WebKitGTK 4.1, GTK 3, libsoup 3, librsvg development packages
  (`libwebkit2gtk-4.1-dev libgtk-3-dev libsoup-3.0-dev librsvg2-dev`)
- Windows: WebView2 runtime (preinstalled on current Windows), MSVC build tools

## Commands

```bash
npm install

# Frontend
npm run dev          # Vite on http://localhost:1420
npm run typecheck    # vue-tsc
npm run build        # typecheck + production build (no harness)
npm test             # Vitest
npm run test:watch

# Core (no webview libraries needed)
cargo test           # default member: forgedesk-core
cargo clippy -p forgedesk-core --all-targets -- -D warnings
cargo fmt --all --check

# Desktop shell (requires Tauri system libraries)
npm run tauri dev
npm run tauri build
```

## UI Gate 0 screenshots

```bash
VITE_VISUAL_HARNESS=true npm run build
npm run preview -- --host 127.0.0.1 --port 4173
PLAYWRIGHT_BROWSERS_PATH=.playwright-browsers node scripts/visual-gate0.mjs
```

Install the browser once:

```bash
PLAYWRIGHT_BROWSERS_PATH=.playwright-browsers npx playwright install chromium
```

The harness is a test fixture only. Production `npm run build` must not include
`Visual harness` in `dist/`; verify with:

```bash
npm run build
grep -R "Visual harness enabled" dist || echo "harness absent"
```

See `docs/ui/visual_baseline.md` for the capture set and native-capture caveat.

## Isolated environments

If `$HOME` is read-only (containers, sandboxes), point package caches inside the
repository:

```bash
export CARGO_HOME="$PWD/.cargo-home"
export CARGO_TARGET_DIR="$PWD/target"
export npm_config_cache="$PWD/.npm-cache"
export PLAYWRIGHT_BROWSERS_PATH="$PWD/.playwright-browsers"
```

All are git-ignored. On a normal workstation leave the defaults.

## Typechecking the Tauri shell without WebKitGTK

The shell needs WebKitGTK to link. `cargo check`/`clippy` only need pkg-config to
find dependency metadata, so in a constrained sandbox temporary `.pc` descriptors
can enable typechecking:

```bash
mkdir -p .tmp/pkgconfig
# webkit2gtk-4.1.pc, javascriptcoregtk-4.1.pc, libsoup-3.0.pc
export PKG_CONFIG_PATH="$PWD/.tmp/pkgconfig"
cargo check -p forgedesk
```

This is a **typecheck workaround**, not a build. It does not prove linking,
runtime webview behavior, or installer correctness. CI runs the real Linux and
Windows shell builds with the proper dependencies.

## Verification notes

- Root `cargo test` intentionally tests only `forgedesk-core` (`default-members`).
  Use `cargo test --workspace` where WebKitGTK/WebView2 is available.
- Native Tauri screenshots/installers must be captured on a machine with the
  actual webview; report them separately from browser-rendered UI gates.
- CI lanes: `.github/workflows/ci.yml` (frontend, core matrix, shell Linux/Windows,
  visual gate artifact).
