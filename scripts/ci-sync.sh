#!/usr/bin/env bash
# Publish the working tree to the Jenkins-readable local mirror.
# This is a local bridge until ForgeDesk has a Git remote.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MIRROR="${FORGEDESK_CI_MIRROR:-$ROOT/.tmp/ci-mirror/forgedesk-ci.git}"
MESSAGE="${1:-CI snapshot}"

if ! git -C "$ROOT" rev-parse --git-dir >/dev/null 2>&1; then
  echo "ForgeDesk is not a git repository. Initialize one before using the CI mirror." >&2
  exit 1
fi

if [ ! -d "$MIRROR" ]; then
  mkdir -p "$(dirname "$MIRROR")"
  git init --bare "$MIRROR" >/dev/null
fi

cd "$ROOT"
if [ -n "$(git status --porcelain)" ]; then
  git add -A
  git commit -m "$MESSAGE"
fi

git push "$MIRROR" HEAD:refs/heads/main
git --git-dir="$MIRROR" update-server-info
echo "Mirror updated: $MIRROR"
