# Stage 6 report — Pull requests, checks and reviews

**Date:** 2026-11-15
**Status:** COMPLETE (DoD) — PR list/detail, commits, files, reviews, checks, review
comments, reviews queue, CI screen, PR↔issue links and guarded review/merge/close are
implemented. Write actions are permission-checked and confirmation-gated but were not
executed against a live repository in CI. Remaining polish: inline comment placement/
thread resolution, PR read budget logs and native Tauri verification.
**Schema:** v6 (`pull_request_link` for PR↔local issue links)

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
- No fake PR data in production; connected screenshots use the build-gated harness.
- Live read-only verification against `cli/cli`: list pulls + load detail/commits/
  reviews/check-runs, review search and CI overview succeeded.
- **Reviews screen:** GitHub search for open PRs with `review-requested:<login>`;
  read-only table opens the corresponding PR detail via `/pull-requests?pr=N`.
- **CI / Activity screen:** default branch, recent commits, check-runs for the latest
  commit, and lazy per-commit check loading; read-only.
- **PR review comments:** read-only comments loaded with the PR detail and shown under
  the matching file with path, line, author and body.
- **Diff modes:** Unified/Split toggle for loaded patches; Split pairs deletions and
  additions with old/new line numbers.
- **PR↔issue links:** schema v6 `pull_request_link`, link/unlink UI, local issue key
  validated in workspace; no remote modification.
- **Permission-aware actions:** merge/squash/rebase with expected head SHA, close,
  and review approve/request-changes/comment. All require explicit confirmation;
  merge/close require admin/maintain/push and are re-checked server-side. No automatic
  retry of ambiguous writes.
- Reliability fixes: idempotent, error-safe GitHub disconnect (memory/keyring
  consistent); PR files pagination follows `Page.next_page` with cycle/non-advance
  rejection and fails the whole request on later-page errors.
- PR Files completeness: separates below-cap incomplete lists from files exceeding the
  3000-file API cap and unknown completeness at the cap; loaded patches stay visible.
- Keyring read failures and Entry creation use the stable `credential_storage` wire
  code. UI distinguishes storage unavailable from auth/network/internal connection-
  check failures, hides "No account connected" unless a successful check confirmed
  absence, and keeps retry plus reauthorization available. Invalid JSON is a safe auth
  error without deletion; memory-only tokens are not re-read from Keyring.

## Not delivered

- Inline review-comment placement and resolved-thread state.
- PR labels/assignees/milestones and merge queue.
- GraphQL batching of review/check summaries for list rows; rows without an open
  detail show `—` for review/checks.
- API-budget observations/request logs for PR read calls.
- Selected repository persistence in SQLite and multi-repository PR scope.
- Native Tauri runtime verification.

## Verification

| Check | Command | Result |
|---|---|---|
| Core tests | `cargo test -p forgedesk-core` | PASS — 78 (+3 ignored live) |
| Frontend | `npm run typecheck`, `npm test` | PASS — 107 |
| Visual gates | `node scripts/visual-gate0.mjs` | PASS — 53 captures (PR, comments, split, merge, reviews, CI) |
| Live PR read | `GITHUB_TOKEN=$(gh auth token) cargo test ... --ignored` | PASS — `cli/cli` list + detail |
| Jenkins build 11 | Stage 6 read-only snapshot `ec5e9dc` | SUCCESS — 144 s, 52 core tests (+2 ignored), 47 captures, 48 artifacts |
| Jenkins build 12 | PR files/diff `3dc3ee1` | SUCCESS — 128 s, 52 core tests (+2 ignored), 48 captures, 49 artifacts |
| Jenkins build 13 | GitHub fixes `8ff8ab4` | SUCCESS — 121 s, 63 core tests (+2 ignored), 48 captures, 49 artifacts |
| GitHub Actions | public repo `main` | SUCCESS (Stage 6 runs) |

## Next

1. Inline comment placement and resolved-thread state.
2. Budget-aware request logging for PR reads.
3. GraphQL list summaries and native Tauri verification.
