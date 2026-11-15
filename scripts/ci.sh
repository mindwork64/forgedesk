#!/usr/bin/env bash
# Full CI gate shared by local runs and Jenkins.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

export CARGO_HOME="${CARGO_HOME:-$ROOT/.cargo-home}"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$ROOT/target}"
export npm_config_cache="${npm_config_cache:-$ROOT/.npm-cache}"
export PLAYWRIGHT_BROWSERS_PATH="${PLAYWRIGHT_BROWSERS_PATH:-$ROOT/.playwright-browsers}"
export CI="${CI:-true}"

mkdir -p "$CARGO_HOME" "$CARGO_TARGET_DIR" "$npm_config_cache" \
  "$PLAYWRIGHT_BROWSERS_PATH" "$ROOT/.tmp"

echo ":: frontend dependencies"
npm ci --no-audit --no-fund

echo ":: frontend typecheck"
npm run typecheck

echo ":: frontend unit tests"
npm test

echo ":: frontend production build"
npm run build

echo ":: rust format"
cargo fmt --all --check

echo ":: core clippy"
cargo clippy -p forgedesk-core --all-targets -- -D warnings

echo ":: core tests"
cargo test -p forgedesk-core

if pkg-config --exists webkit2gtk-4.1 \
  && pkg-config --exists javascriptcoregtk-4.1 \
  && pkg-config --exists libsoup-3.0; then
  echo ":: tauri shell typecheck (real WebKitGTK pkg-config)"
else
  echo ":: tauri shell typecheck (ci/pkgconfig typecheck shims; no native link)"
  export PKG_CONFIG_PATH="$ROOT/ci/pkgconfig${PKG_CONFIG_PATH:+:$PKG_CONFIG_PATH}"
fi
cargo check -p forgedesk
cargo clippy -p forgedesk --all-targets -- -D warnings

echo ":: playwright browser"
npx playwright install chromium >/dev/null

echo ":: UI Gate 0"
VITE_VISUAL_HARNESS=true npm run build
PREVIEW_PORT="${FORGEDESK_PREVIEW_PORT:-4173}"
npm run preview -- --host 127.0.0.1 --port "$PREVIEW_PORT" \
  > "$ROOT/.tmp/preview.log" 2>&1 &
PREVIEW_PID=$!
cleanup() {
  kill "$PREVIEW_PID" 2>/dev/null || true
}
trap cleanup EXIT

for _ in $(seq 1 30); do
  if curl -fsS "http://127.0.0.1:$PREVIEW_PORT/" >/dev/null 2>&1; then
    break
  fi
  sleep 1
done
curl -fsS "http://127.0.0.1:$PREVIEW_PORT/" >/dev/null
PREVIEW_URL="http://127.0.0.1:$PREVIEW_PORT" node scripts/visual-gate0.mjs
cleanup
trap - EXIT

echo ":: production build after visual gate"
npm run build
if grep -R "Visual harness enabled" dist >/dev/null 2>&1; then
  echo "Visual harness leaked into the production build" >&2
  exit 1
fi

echo ":: CI OK"
