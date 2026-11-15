# ADR-0009 — Capability-driven GitLab/Gitea adapters

**Date:** 2026-11-15
**Status:** accepted
**Stage:** 8

## Decisions

1. **One normalized contract, provider-specific transports.**
   - `ProviderAdapter` exposes repositories, issues and merge requests with
     provider-neutral DTOs and explicit capabilities.
2. **Capabilities are declared, not inferred from provider names.**
   - GitLab/Gitea currently advertise read-only repositories/issues/MRs.
   - Comments, reviews, checks and writes are false until implemented; UI must show
     limitations.
3. **No live host is assumed.**
   - Base URLs/tokens are constructor inputs; tests use local mock servers.
   - UI shows read-only adapter ready and credentials not configured; it does not fake
     a connection.
4. **Pagination is provider-specific.**
   - GitLab uses `X-Next-Page`; Gitea uses `X-HasMore` and returns `page + 1`.
5. **No provider data is written to SQLite in this stage.**
   - Persistence/linking/sync integration is future work; this avoids inventing an
     unverified remote ownership model.

## Consequences

- Adapters can be contract-tested without network credentials.
- A live verification remains a release task when hosts/tokens exist.
- GitLab/Gitea write and review capabilities stay visibly unsupported.
