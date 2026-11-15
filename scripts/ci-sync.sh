#!/usr/bin/env bash
# Publish the working tree to the Jenkins-readable local mirror.
# This is a local bridge until ForgeDesk has a Git remote.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MIRROR="${FORGEDESK_CI_MIRROR:-/tmp/forgedesk-ci.git}"
MESSAGE="${1:-CI snapshot}"

if ! git -C "$ROOT" rev-parse --git-dir >/dev/null 2>&1; then
  echo "ForgeDesk is not a git repository. Initialize one before using the CI mirror." >&2
  exit 1
fi

if [ ! -d "$MIRROR" ]; then
  git init --bare "$MIRROR" >/dev/null
fi

cd "$ROOT"
if [ -n "$(git status --porcelain)" ]; then
  git add -A
  git commit -m "$MESSAGE"
fi

git push "$MIRROR" HEAD:refs/heads/main
echo "Mirror updated: $MIRROR"
