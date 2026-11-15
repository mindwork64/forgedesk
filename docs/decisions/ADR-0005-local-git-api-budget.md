# ADR-0005 — Local Git, provider credentials and API budget foundation

**Date:** 2026-11-15  
**Status:** accepted  
**Stage:** 4

## Context

Stage 4 introduces OS Git processes, a repository registry, GitHub connection
foundation and API budget/queue persistence. The app must stay offline-first and
must never fake a provider connection or ship secrets.

## Decisions

1. **Git is invoked directly, never through a shell.**
   - `Command::new("git").args([...]).current_dir(path)` with
     `GIT_TERMINAL_PROMPT=0`. Values are separate argv entries; no `sh -c`,
     PowerShell concatenation or interpolation.
   - Branches are validated by `git check-ref-format --branch` plus newline/NUL/
     leading-dash checks. Dirty worktrees block create/switch.
2. **Only user-attached local repositories exist in Stage 4.**
   - `repository` stores canonical path, provider, remote URL, default branch and
     optional project. GitHub discovery requires auth and is deferred.
3. **Provider credentials are not stored in SQLite.**
   - `provider_account` keeps identity/host/scopes/status only; no token columns.
   - Until a secure OS keychain implementation exists, GitHub connect is disabled
     and the UI says credentials are unavailable. No fake account state.
4. **API budget observations preserve provenance.**
   - `limit/remaining/used/reset/observed/source/is_estimate/secondary_pause`
     distinguish GitHub-reported values from local observations.
   - Local request logs store sanitized operation metadata, not Authorization
     headers, tokens, private bodies or diffs.
5. **Background allowance uses the stricter bound.**
   - `min(10% × observed limit, max(0, remaining − 20% × limit))`; unknown values
     yield zero allowance. Interactive work is not silently treated as background.
6. **Queue and control are persisted before a dispatcher exists.**
   - `api_queue_job` and `api_control` are the durable foundation for the Stage 4/5
     dispatcher. No worker is simulated; the queue is empty until a provider client
     exists.
7. **API Monitor is first-class and honest.**
   - Empty/unknown states are rendered explicitly; no percentage gauge implies exact
     quota. Screenshots with connected budgets use the build-gated harness only.
8. **No provider write in Stage 4.**
   - GitHub auth/network is deferred; no token was supplied and no external write is
     performed.

## Consequences

- Local Git is testable offline and safe against argument injection.
- Removing a repository row never deletes files or branches.
- A future auth implementation must add keychain/OS credential storage, device-flow
  polling rules, token refresh and revocation before enabling Connect.
- The dispatcher must consume `api_queue_job` and write `api_request_log`/
  observations; adding HTTP adapters must not bypass it.
- API Monitor can ship now with useful local state and honest empty provider state.

## Rejected alternatives

- **`sh -c "git ..."`** — injection and quoting risk.
- **Storing tokens in SQLite** — local DB leaks file copies/backups.
- **Fake GitHub connect for screenshots in production** — violates no-fake-UI rule.
- **Automatic branch switching on dirty worktree** — data-loss risk.
- **Counting `limit − remaining` as ForgeDesk usage** — other apps consume quota.
- **Live background polling without budgets** — uncontrolled API spend.
