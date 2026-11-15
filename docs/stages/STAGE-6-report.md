# Stage 6 report — Pull requests, checks and reviews

**Date:** 2026-11-15  
**Status:** PARTIAL — read-only pull-request list and detail (overview, commits,
checks, reviews, unified file patches) are delivered and live-tested. Split diffs,
line comments, review submission, merge/close actions, requested-review queue and CI
screen remain open.  
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
  - Reviews: reviewer, state and body;
  - Files: changed files with status/additions/deletions and unified patches.
- No merge, close, approve, comment or review submission is exposed.
- No fake PR data in production; connected screenshots use the build-gated harness.
- Live read-only verification against `cli/cli`: list pulls + load detail/commits/
  reviews/check-runs succeeded.
- Reliability fixes: idempotent, error-safe GitHub disconnect (memory/keyring
  consistent); PR files pagination follows `Page.next_page` with cycle/non-advance
  rejection and fails the whole request on later-page errors.
- PR Files completeness: compares loaded files with `changed_files` and the 3000-file
  API cap; warns for known-missing or indeterminate results without hiding patches.
- Keyring read failures/invalid JSON return safe auth errors, keep state, and surface
  as storage-unavailable (not disconnected) with retry and reauthorization available.
  Memory-only tokens are not re-read from Keyring.

## Not delivered

- Split diff, line comments and resolved-thread state.
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
| Core tests | `cargo test -p forgedesk-core` | PASS — 69 (+2 ignored live) |
| Frontend | `npm run typecheck`, `npm test` | PASS — 89 |
| Visual gates | `node scripts/visual-gate0.mjs` | PASS — 48 captures (PR list/detail/files) |
| Live PR read | `GITHUB_TOKEN=$(gh auth token) cargo test ... --ignored` | PASS — `cli/cli` list + detail |
| Jenkins build 11 | Stage 6 read-only snapshot `ec5e9dc` | SUCCESS — 144 s, 52 core tests (+2 ignored), 47 captures, 48 artifacts |
| Jenkins build 12 | PR files/diff `3dc3ee1` | SUCCESS — 128 s, 52 core tests (+2 ignored), 48 captures, 49 artifacts |
| Jenkins build 13 | GitHub fixes `8ff8ab4` | SUCCESS — 121 s, 63 core tests (+2 ignored), 48 captures, 49 artifacts |
| GitHub Actions | public repo `main` | SUCCESS (Stage 6 runs) |

## Next

1. Split diff, line comments and lazy per-file diff loading.
2. Review submission, merge/close and permission checks with explicit intent.
3. `/reviews` requested queue and `/ci` checks screen.
4. PR↔issue links and GraphQL list summaries.
5. Budget-aware request logging for PR reads.
6. Native Tauri verification.
