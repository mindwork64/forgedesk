# ADR-0003 — Issue model, workflow and filter engine

**Date:** 2026-11-15  
**Status:** accepted  
**Stage:** 2

## Context

Stage 2 introduces the first large domain surface: issues, workflow statuses,
labels, hierarchy, relations, comments, activity, search and filtered list views.
The schema is append-only, so the core relations and ownership rules need to be
durable now even though Projects/Cycles UI arrives later.

## Decisions

1. **One issue table with explicit columns, not a JSON blob.**
   - `issue` stores title, description, status FK, priority, type, estimate,
     start/due dates, project FK, parent FK, source, remote URL, archive/delete
     timestamps and lifecycle timestamps.
   - Foreign keys and check constraints keep enums valid at the database boundary.
2. **Workflow per workspace with seeded semantic defaults.**
   - `workflow` + `workflow_status`; statuses carry category, color token, order
     and an optional `system_key` (`backlog`, `todo`, `in_progress`, `done`,
     `canceled`).
   - Default status names are localized in the UI through `system_key`; custom
     names remain user data. Position ordering is explicit.
   - Workflow editing/transitions are deliberately not exposed yet; the model
     supports them without migration.
3. **Human keys are allocated transactionally.**
   - `workspace_key_counter` is incremented inside the create-issue transaction;
     the display key is `FD-<n>`. A unique `(workspace_id, key_number)` and
     `(workspace_id, key)` prevents duplicates.
4. **Hierarchy is a self-reference with cycle prevention.**
   - Parent changes walk the ancestor chain inside the transaction and reject
     self/descendant/cyclic links. Arbitrary depth is allowed.
5. **Relations are directed rows with derived perspective.**
   - `blocks`, `related` and `duplicate` are stored once; `blockedBy` and
     `duplicatedBy` are derived for the opposite side. `related` is canonicalized
     by id order to prevent mirror duplicates.
6. **Activity is append-only structured data.**
   - `activity_event(kind, data JSON)` records creation, status, field, label,
     relation, archive and duplicate events. Only field names/old/new status names
     are stored, not private body text.
7. **Filtering is server-side and whitelisted.**
   - `IssueQuery` supports an AND/OR tree (one nested group level), field/operator
     whitelists, parameterized values, FTS search, sort whitelist and limit/offset.
   - No user string is interpolated into SQL. `LIKE` input is escaped; FTS input is
     tokenized into quoted terms. The built SQL and params are unit tested,
     including an injection-shaped title filter.
8. **Search uses FTS5 with triggers.**
   - `issue_fts(issue_id, title, description)` is kept in sync by insert/update/
     delete triggers. Search is local; provider search remains an explicit future
     action with API cost.
9. **Archive is soft.**
   - `archived_at` removes an issue from active lists; descendants remain linked.
     No cascade delete is exposed in the UI.
10. **Projects are a minimal FK target in Stage 2.**
    - The Projects screen and full lifecycle are Stage 3. A small `project` table
      allows issue assignment/filtering and tests now; creation is not exposed as
      a fake UI.
11. **Stage 3 columns stay additive.**
    - Milestones, cycles, docs and attachments will be added by new migrations
      with nullable FKs rather than rewriting the issue table.
12. **Saved views store the query, not issue ids.**
    - Name, filter tree, sort, groupBy, columns, density, pin and position are
      persisted per workspace. Applying a view re-runs the query, so results stay
      current as issues change. Pinning only affects the sidebar order.
13. **The client never builds filter SQL.**
    - Vue sends the typed filter tree and receives summaries/details. The backend
      validates field/operator combinations and rejects unsupported ones.

## Consequences

- The schema is explicit and queryable; provider-neutral remote IDs stay opaque
  strings separate from local IDs.
- Saved views persist the same typed filter/sort/groupBy shape in `saved_view`;
  columns/density are stored for a later column chooser, while the current list
  renders the fixed contract columns.
- Status renaming/localization is possible without changing issue rows.
- Workflow transitions, templates, watchers and attachments still need product
  UI/policy work; they are not simulated.
- Projects/Cycles schema work in Stage 3 must preserve the issue ownership rules
  and key counter.

## Rejected alternatives

- **JSON issue payload** — loses constraints, indexes and safe filtering.
- **Client-side filtering only** — cannot scale to hundreds/thousands of issues and
  would expose inconsistent semantics.
- **Dynamic SQL by column-name interpolation** — injection and correctness risk.
- **Global numeric issue IDs** — provider keys are repository-scoped; local keys
  must remain stable and human readable.
- **Hard delete on archive** — irreversible data loss in a single-user local app.
- **Seeding every Stage 3 table now** — premature schema without UI/policy, higher
  migration cost.
