# ADR-0008 — PR↔issue links and guarded pull-request actions

**Date:** 2026-11-15
**Status:** accepted
**Stage:** 6 completion

## Context

Stage 6 requires linking pull requests to local tasks and safely performing supported
review/merge actions. GitHub remote writes are irreversible and can be ambiguous on
timeouts.

## Decisions

1. **Links are local-only in schema v6.**
   - `pull_request_link` maps provider/account/repository/pull number to a local issue.
   - Linking/unlinking never modifies GitHub.
   - The issue key is resolved inside the current workspace; unknown keys fail.
2. **Remote actions require explicit confirmation.**
   - Commands reject `confirmed = false`: merge, close, and review submission.
   - The UI opens a confirmation dialog; there is no background/automatic action.
3. **Permissions are re-checked server-side.**
   - Merge/close require repository `admin`, `maintain`, or `push` permissions.
   - Reviews require pull/triage/write access; approving one's own PR is rejected.
   - UI hides/disables actions when capabilities cannot be loaded.
4. **Merge is head-pinned.**
   - The current head SHA is sent to GitHub to avoid merging a newer commit than the
     one shown in the UI.
5. **No automatic retry of ambiguous writes.**
   - A timeout/network failure is surfaced; ForgeDesk does not replay the mutation.
   - The user can refresh and retry intentionally.
6. **Write verification is not performed in CI.**
   - Tests cover validation, permissions, client parsing and UI confirmation without
     creating real PRs/merges/reviews.

## Consequences

- PR/task history survives locally and can be queried without GitHub.
- A user with stale UI data may get a merge conflict/validation error; head pinning
  prevents silent newer-commit merges.
- Outbox/reconciliation for PR writes is future work; direct calls are acceptable for
  explicit user-triggered actions but not background sync.
- Live end-to-end write testing remains a release/manual-verification task.
