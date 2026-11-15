#!/usr/bin/env bash
# Trigger the ForgeDesk Jenkins job. Credentials come from the environment.
set -euo pipefail

: "${JENKINS_URL:?Set JENKINS_URL}"
: "${JENKINS_USER:?Set JENKINS_USER}"
: "${JENKINS_TOKEN:?Set JENKINS_TOKEN}"
JOB="${JENKINS_JOB:-ForgeDesk-CI}"

crumb_json="$(curl -g -sS --compressed -u "$JENKINS_USER:$JENKINS_TOKEN" "$JENKINS_URL/crumbIssuer/api/json")"
crumb="$(printf '%s' "$crumb_json" | python3 -c 'import sys,json; print(json.load(sys.stdin)["crumb"])')"
crumb_field="$(printf '%s' "$crumb_json" | python3 -c 'import sys,json; print(json.load(sys.stdin)["crumbRequestField"])')"

code="$(curl -g -sS --compressed -o /dev/null -w '%{http_code}' \
  -u "$JENKINS_USER:$JENKINS_TOKEN" \
  -H "$crumb_field: $crumb" \
  -X POST "$JENKINS_URL/job/$JOB/build")"

if [ "$code" != "201" ]; then
  echo "Jenkins returned HTTP $code while queuing $JOB" >&2
  exit 1
fi
echo "Queued $JOB"
