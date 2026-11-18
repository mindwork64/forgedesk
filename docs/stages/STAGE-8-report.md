## Full-product status addendum — 2026-11-18

Stage 8 is currently a fixture/mock-tested, read-only GitLab/Gitea adapter foundation. It is not yet the live basic provider support described by 01_PRODUCT_PROJECT.md.

Remaining: secure host/account connection UI, OS keyring token handling, persisted provider identity, repository/Issue/MR discovery, supported branches/commits/labels/comments/releases/pipelines and safe writes where capability/permission allow. Live test against each provider when host/credentials are available; otherwise mark NOT VERIFIED. See PRODUCT_COMPLETION_SPEC.md §4.8.

---

# Stage 8 report — GitLab/Gitea provider adapters

**Date:** 2026-11-15
**Status:** COMPLETE (fixtures) — capability-driven read-only GitLab and Gitea
adapters are implemented and contract-tested against local mock servers. No live
GitLab/Gitea host or credentials were available, so live integration tests were not
run; the UI states this explicitly.
**Schema:** v9 (no new provider tables)

## Delivered

- `ProviderAdapter` contract with normalized `RemoteRepository`, `RemoteIssue`,
  `RemoteMergeRequest`, `ProviderPage<T>` and explicit `ProviderCapabilities`.
- GitLab adapter:
  - `GET /api/v4/projects` (membership, simple);
  - project issues and merge requests by encoded project path;
  - `X-Next-Page` pagination;
  - `PRIVATE-TOKEN` auth.
- Gitea adapter:
  - `/api/v1/user/repos`, repo issues and pulls;
  - PR entries filtered out of issue lists;
  - `X-HasMore` pagination with `page + 1`;
  - `Authorization: token` auth.
- Capabilities for both: repositories, issues, merge requests, read-only; comments,
  reviews, checks and writes are explicitly false.
- UI Integrations cards show “Read-only adapter ready”, “Credentials not configured”
  and the supported capability list. No fake connection state.
- Contract tests with local mock HTTP servers validate normalization, pagination and
  PR filtering for both providers.

## Not delivered

- Live GitLab/Gitea host verification (no host/token available).
- Provider connection UI/token vault flows for GitLab/Gitea.
- Issue/MR write, comments, reviews, checks and provider-specific sync cursors.
- GitLab/Gitea persistence/linking beyond the adapter contract.
- Native Tauri verification.

## Verification

| Check | Command | Result |
|---|---|---|
| Core tests | `cargo test -p forgedesk-core` | PASS — 90 (+3 ignored live) |
| Provider contract tests | `cargo test -p forgedesk-core provider` | PASS — 3 |
| Frontend | `npm run typecheck`, `npm test` | PASS — 107 |
| Visual gates | `node scripts/visual-gate0.mjs` | PASS — 57 captures |
| Live GitLab/Gitea | — | NOT RUN — no host/token |

## Next

Stage 9 optional AI extension; remaining Stage 5/6 polish (sync writes, native
verification) stays tracked.
