# Stage 6 report — Pull requests, checks and reviews

**Date:** 2026-11-15  
**Status:** PARTIAL — read-only pull-request list and detail (overview, commits,
checks, reviews) are delivered and live-tested. File diffs, review submission,
merge/close actions, requested-review queue and CI screen remain open.  
**Schema:** v5 (no new local tables for Stage 6 yet)

## Delivered

- GitHub REST client methods:
  - list pulls (open/closed/all, 50/page, sorted by update);
  - pull detail;
  - pull commits;
  - pull reviews;
  - check runs for the head SHA.
- Repository context: a discovered GitHub repository can be selected for pull
  requests; the selection is retained in WebView storage (not SQLite yet).
- **Pull Requests screen:**
  - empty states for disconnected GitHub and no selected repository;
  - Open/Closed/All segmented filter, refresh;
  - table: PR number/title/author, repository, state, draft, branches, updated;
  - 440px detail pane with Overview / Commits / Checks / Reviews tabs;
  - Overview: description, source→target branch, additions/deletions/files,
    mergeable, requested reviewers;
  - Commits: SHA, message, author;
  - Checks: status/conclusion and duration;
  - Reviews: reviewer, state and body.
- No merge, close, approve, comment or review submission is exposed.
- No fake PR data in production; connected screenshots use the build-gated harness.
- Live read-only verification against `cli/cli`: list pulls + load detail/commits/
  reviews/check-runs succeeded.

## Not delivered

- File list/diff viewer (unified/split), line comments and resolved state.
- Review submission, approve/request-changes, merge/close, merge readiness and
  permission-aware actions.
- Requested-review queue (`/reviews`) and CI/checks (`/ci`) dedicated screens.
- PR-to-local-issue linking, PR labels/assignees/milestones, merge queue.
- GraphQL batching of review/check summaries for list rows; rows without an open
  detail show `—` for review/checks.
- API-budget observations/request logs for PR read calls.
- Selected repository persistence in SQLite and multi-repository PR scope.
- Native Tauri runtime verification.

## Verification

| Check | Command | Result |
|---|---|---|
| Core tests | `cargo test -p forgedesk-core` | PASS — 52 (+2 ignored live) |
| Frontend | `npm run typecheck`, `npm test` | PASS — 78 |
| Visual gates | `node scripts/visual-gate0.mjs` | PASS — 47 captures (PR list + detail added) |
| Live PR read | `GITHUB_TOKEN=$(gh auth token) cargo test ... --ignored` | PASS — `cli/cli` list + detail |
| Jenkins build 11 | Stage 6 read-only snapshot | Pending commit/mirror push |
| GitHub Actions | public repo | Pending push |

## Next

1. PR Files tab with lazy diff and unified/split.
2. Review submission, merge/close and permission checks with explicit intent.
3. `/reviews` requested queue and `/ci` checks screen.
4. PR↔issue links and GraphQL list summaries.
5. Budget-aware request logging for PR reads.
6. Native Tauri verification.
