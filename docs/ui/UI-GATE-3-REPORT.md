# UI Gate 3 acceptance report — Repositories, Integrations and API Monitor

**Gate:** GitHub screens and API Monitor (`06_UI_UX_SPEC.md` §12)  
**Stage:** 4  
**Date:** 2026-11-15  
**Verdict:** PARTIAL — Local Git, Integrations and API Monitor pass browser review for
the delivered local/foundation scope. PR/Review/CI screens are Stage 6; live GitHub
data requires Stage 4 authentication, which is not configured.

## Delivered

- Repositories list with dense table, local provider, default branch, path, linked
  project, last checked, empty/error/loading states and Git availability banner.
- Add local repository dialog (path/name/project) with backend canonicalization and
  duplicate/path validation.
- Repository detail with Overview/Branches/Recent commits, dirty/clean badge,
  ahead/behind/upstream, diff stat, branch list with switch actions and guarded
  branch creation.
- Integrations: GitHub primary card with real Device Flow connect/reauthorize/
  refresh/disconnect, connection state and readable read/write scopes; Local Git
  real state/version/repository count; GitLab/Gitea planned.
- GitHub Device Flow dialog with device code, verification URL, automatic polling,
  copy actions and keyring/memory storage state.
- GitHub repository discovery + read-only issue import dialog (up to 250 recently
  updated issues, idempotent remote links; no comments or GitHub writes).
- Read-only Reviews queue and CI/Activity screens for the selected repository.
- Pull Requests read-only table and 440px detail pane (Overview/Commits/Checks/
  Reviews/Files) for the selected discovered repository; incomplete and 3000-file
  cap warnings preserve visible patches.
- API Monitor: connection/scheduler/dispatcher/last observation/reset summary,
  secondary-limit state, GraphQL/REST primary panels, resource buckets, background
  allowance/reserve, queue, recent sanitized requests, manual pause/resume and
  diagnostics export.
- Explicit labels separate GitHub-reported remaining from ForgeDesk-observed
  requests; unknown values render as `—`, not a fake gauge.

## Captures (Gate 3 subset)

| # | File | Viewport | State |
|---|---|---|---:|
| 1 | `gate3-repositories-dark-1440.png` | 1440×900 | local repositories table |
| 2 | `gate3-repository-detail-dark-1440.png` | 1440×900 | clean repo overview/branches |
| 3 | `gate3-repository-dirty-dark-1440.png` | 1440×900 | dirty worktree + guard |
| 4 | `gate3-integrations-dark-1440.png` | 1440×900 | GitHub/Local/GitLab/Gitea cards |
| 5 | `gate3-api-monitor-dark-1440.png` | 1440×900 | budgets, queue, request log |
| 6 | `gate3-github-auth-dark-1440.png` | 1440×900 | Device Flow dialog with code |
| 7 | `gate3-github-repos-dark-1440.png` | 1440×900 | discovered GitHub repos + import |
| 8 | `gate3-prs-dark-1440.png` | 1440×900 | PR table with Open/Closed/All |
| 9 | `gate3-pr-detail-dark-1440.png` | 1440×900 | PR detail tabs/reviews/checks |
| 10 | `gate3-pr-files-dark-1440.png` | 1440×900 | unified PR file patches |
| 11 | `gate3-pr-files-overlimit-dark-1440.png` | 1440×900 | 3000-file API cap warning |
| 12 | `gate3-reviews-dark-1440.png` | 1440×900 | requested reviews queue |
| 13 | `gate3-ci-dark-1440.png` | 1440×900 | commits + check-runs |

Full run: **51 captures** (16 Gate 0 + 13 Gate 1 + 9 Gate 2 + 13 Gate 3),
`docs/ui/current/measurements.json`, zero assertion failures.

## Acceptance checklist

| Check | Result | Evidence |
|---|---|---|
| Repositories list, ≥2 local repos | PASS | provider Local, branch/path/project |
| Add local repository validation | PASS backend tests | duplicate/path canonicalization |
| Repository detail branches/commits | PASS | 3 branches + commit log |
| Dirty worktree guard visible | PASS | dirty badge, disabled switch, warning |
| Integrations GitHub primary with scopes | PASS | real Connect + read/write lists |
| GitHub Device Flow code/polling UI | PASS | code/URL, auto-poll, keyring/memory state |
| Live token acquisition + refresh | PASS (manual) | `/user` 200, refresh 200, new token pair |
| API Monitor primary budgets/resource | PASS with fixtures | 2 primary panels + search bucket |
| API Monitor queue/request history | PASS with fixtures | 2 queue jobs, 3 requests |
| Pause/resume background sync | PASS | persisted api_control |
| Diagnostics export sanitized | PASS | JSON copy, no tokens/payloads |
| GitHub repository discovery | PASS | real `user/repos`, selection for PRs |
| Pull request read-only list/detail | PASS | live `cli/cli` list + detail |
| PR files/diff and write actions | NOT implemented | next Stage 6 slice |
| PR/Reviews/CI screens | NOT in Stage 4 | Stage 6 |
| Native Tauri capture | SKIPPED | no WebKitGTK runtime |

## Composition notes

- Repository detail uses fixed local tabs and keeps Repositories active in sidebar.
- API Monitor is a first-class screen, not a header widget; budgets are panels, not
  guarantees, and other-app usage is called out.
- Connected/budget data in Gate 3 screenshots comes from the build-gated harness;
  production shows the explicit empty/not-connected states.
- No provider write occurred during implementation or verification.

## Conclusion

The delivered Stage 4–6 read-only screens pass browser visual acceptance for local
Git, API budgets, GitHub auth/import and pull requests. The gate remains **PARTIAL**
until PR files/write actions, dispatcher, provider-backed API Monitor history and
native Tauri verification exist.
