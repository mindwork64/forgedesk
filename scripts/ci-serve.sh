#!/usr/bin/env bash
# Serve the local CI mirror read-only over git:// for Jenkins.
# Requires the mirror to exist (scripts/ci-sync.sh).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MIRROR_ROOT="${FORGEDESK_CI_MIRROR_ROOT:-$ROOT/.tmp/ci-mirror}"
PORT="${FORGEDESK_CI_GIT_PORT:-9418}"

if [ ! -d "$MIRROR_ROOT/forgedesk-ci.git" ]; then
  echo "CI mirror missing; run scripts/ci-sync.sh first." >&2
  exit 1
fi

exec git daemon \
  --reuseaddr \
  --export-all \
  --base-path="$MIRROR_ROOT" \
  --listen=127.0.0.1 \
  --port="$PORT" \
  --verbose
