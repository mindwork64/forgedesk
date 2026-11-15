# ADR-0006 — Sync links, outbox safety and conflict ownership

**Date:** 2026-11-15  
**Status:** accepted  
**Stage:** 5 foundation

## Context

GitHub issue synchronization must never duplicate creates/updates when a network
outcome is ambiguous. Provider authentication is not available yet, but the durable
local model and state machine can be validated offline now.

## Decisions

1. **Remote links are one-to-one with local issues.**
   - `remote_issue_link.local_issue_id` is unique; provider account/repository/remote
     id are stored separately from local UUIDs. Remote numbers are repository-scoped.
2. **Field ownership is explicit JSON, not inferred.**
   - Each linked issue carries which fields are local-owned, remote-owned or shared;
     conflict resolution updates this map. No silent last-write-wins.
3. **Outbox is persisted before dispatch.**
   - Unique idempotency key; duplicate enqueue returns the existing job.
   - Allowed transitions are a closed state machine. `outcome_unknown` cannot be
     retried or marked succeeded directly; it must enter `reconciling`.
4. **Ambiguous mutations are reconciled, never auto-replayed.**
   - The dispatcher must first read/search for the remote object or state, then mark
     `succeeded`, `retry_safe` or `needs_user_action`.
5. **Cursors are per account/resource/scope.**
   - Page cursor and time watermark are separate; `last_success_at` is not advanced
     on partial or failed pages; `incomplete_reason` is retained.
6. **Conflicts are first-class rows.**
   - Local/remote values and remote update time are retained; resolution is explicit.
     The UI must show unresolved conflicts before allowing overwrite.
7. **No provider call before auth/budget infrastructure.**
   - The schema and state machine are inert until Stage 4 auth/client exists. No fake
     sync controls are rendered.

## Consequences

- The safety-critical part can be tested offline and reviewed before network code.
- Live dispatcher must use the outbox table as its only write path.
- A future “multiple remote links per issue” change requires a migration from the
  unique local issue constraint.
- Conflict ownership needs a user-facing resolution UI before Stage 5 can be called
  complete.

## Rejected alternatives

- **Retry every failed mutation** — duplicates after timeouts.
- **Last-write-wins without ownership** — silent data loss.
- **Remote IDs in the local issue primary key** — provider-specific and unstable.
- **In-memory outbox** — loses ambiguous state on app exit.
- **Fake sync UI before auth** — violates no-fake-product rule.
