#!/usr/bin/env bash
# Create or update the local Jenkins pipeline job from ci/jenkins/ForgeDesk-CI.xml.
# Credentials are read from the environment and are never written to disk.
set -euo pipefail

: "${JENKINS_URL:?Set JENKINS_URL, e.g. http://127.0.0.1:8080}"
: "${JENKINS_USER:?Set JENKINS_USER}"
: "${JENKINS_TOKEN:?Set JENKINS_TOKEN}"

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
JOB="${JENKINS_JOB:-ForgeDesk-CI}"
CONFIG="$ROOT/ci/jenkins/ForgeDesk-CI.xml"

crumb_json="$(curl -g -sS --compressed -u "$JENKINS_USER:$JENKINS_TOKEN" "$JENKINS_URL/crumbIssuer/api/json")"
crumb="$(printf '%s' "$crumb_json" | python3 -c 'import sys,json; print(json.load(sys.stdin)["crumb"])')"
crumb_field="$(printf '%s' "$crumb_json" | python3 -c 'import sys,json; print(json.load(sys.stdin)["crumbRequestField"])')"

if curl -g -fsS --compressed -u "$JENKINS_USER:$JENKINS_TOKEN" \
  "$JENKINS_URL/job/$JOB/api/json" >/dev/null 2>&1; then
  endpoint="$JENKINS_URL/job/$JOB/config.xml"
  action="updated"
else
  endpoint="$JENKINS_URL/createItem?name=$JOB"
  action="created"
fi

code="$(curl -g -sS --compressed -o /dev/null -w '%{http_code}' \
  -u "$JENKINS_USER:$JENKINS_TOKEN" \
  -H "$crumb_field: $crumb" \
  -H 'Content-Type: application/xml' \
  --data-binary "@$CONFIG" \
  "$endpoint")"

if [ "$code" != "200" ] && [ "$code" != "201" ]; then
  echo "Jenkins returned HTTP $code while creating/updating $JOB" >&2
  exit 1
fi
echo "Jenkins job $JOB $action"
