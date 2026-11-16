# ADR-0011 — Logical repository, checkout and project association

**Date:** 2026-11-16
**Status:** accepted
**Stage:** 10 repository correction

## Decision

- `repository` is a logical provider or local-only entity. It has stable provider
  identity (`provider`, `host`, `external_id`), provider metadata and a normalized
  canonical remote URL. It has no local path.
- `repository_checkout` stores one optional local clone per logical repository. A
  repository may have zero or one checkout; a canonical path is unique per workspace.
  Detaching a checkout removes only that row and never deletes files or the repository
  entity. Multiple clones per remote are intentionally out of scope for v1.
- `repository_project` is the sole source of project↔repository truth (many-to-many).
  The legacy `repository.project_id` was migrated once and is no longer read.
- Legacy duplicate checkouts of one origin cannot be represented under the one-checkout
  rule. Migration keeps the first as the provider entity and preserves later rows as
  explicit local-only conflicts with an `identity_conflict` diagnostic; no row, path or
  project link is silently discarded.
- GitHub discovery upserts by stable provider ID, falling back to canonical remote URL.
  A local-only row can be reconciled when discovery supplies provider identity.
- Import fetches GitHub data before opening the SQLite apply transaction. Repository
  identity and project association are created in that atomic apply phase.
- PR repository selection is stored in SQLite `app_setting` by repository ID and
  hydrated to current metadata; localStorage is only a compatibility cache.

## Consequences

- Remote-only repositories are valid and cannot execute Git operations until a
  checkout is attached.
- Renames preserve internal IDs, checkouts, project links and issue links.
- Unlink/detach operations do not delete Issues, remote links or local files.
- AppImage/OS packaging and live GitHub verification remain separate release gates.
