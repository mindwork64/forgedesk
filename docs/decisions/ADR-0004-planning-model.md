# ADR-0004 — Planning model: projects, initiatives, milestones, cycles, documents

**Date:** 2026-11-15  
**Status:** accepted  
**Stage:** 3

## Context

Stage 3 adds planning above the issue workflow: project/initiative hierarchy,
milestones, work cycles, documents and progress. Stage 2 already created a minimal
`project` table used as an issue FK. The schema is append-only and must survive
provider sync and later roadmap/automation work.

## Decisions

1. **Project is the existing Stage 2 table, extended in place.**
   - Migration v3 adds `initiative_id`, `color`, `progress_method`; status,
     priority, dates and archive already existed.
   - Issues keep `project_id`; no second issue↔project relation table.
2. **Initiative is a separate table linked by `project.initiative_id`.**
   - One project belongs to at most one initiative in v1. This prevents duplicate
     aggregation in the single-user model.
3. **Milestones belong to one project.**
   - `milestone.project_id` is required; due date/status/archive are local.
4. **Cycles and milestones link to issues via one-row join tables.**
   - `issue_cycle(issue_id PRIMARY KEY, cycle_id)` and
     `issue_milestone(issue_id PRIMARY KEY, milestone_id)`.
   - An issue can be in at most one cycle and at most one milestone in v1. This
     keeps the relationship unambiguous and avoids altering the hot issue summary
     query for every Stage 3 screen.
5. **Progress uses one documented formula.**
   - `completed / (total − canceled)`, clamped to `[0, 1]`, one decimal percent.
   - `total` excludes archived/deleted issues; canceled is excluded from the
     denominator.
   - Projects: issues with `project_id`. Milestones: linked issues. Initiatives:
     weighted sum of linked projects, not an average of project percentages.
6. **Cycle status is derived, never stored.**
   - Compare start/end against the local date: `upcoming`, `active`, `completed`.
   - Summary counts/estimates/carry-over are computed at read time from linked
     issues; no stale denormalized counters.
7. **Documents are Markdown source, not a block editor.**
   - `title`, `body`, optional project/initiative links, timestamps, archive flag.
   - Export wraps the body in `# title`; the UI copies the exported Markdown.
8. **No provider data in Stage 3 planning tables.**
   - Repositories/PR summaries remain Stage 4+; the overview shows an honest empty
     state rather than seeded link data.
9. **Templates are deliberately deferred.**
   - They need product decisions about field defaults and versioning; no fake
     template table/UI is introduced.
10. **Roadmap is read-only in v1.**
    - Projects/initiatives with dates are plotted; drag editing and dependencies
      require an explicit edit mode and are deferred.

## Consequences

- Progress is explainable and testable; the same formula is used in backend tests
  and UI fixtures.
- Issue summary queries are unchanged; cycle/milestone screens use dedicated joins.
- A future “multiple cycles per issue” change requires a migration from primary-key
  join rows to a history table.
- Project/initiative archives hide from lists but do not cascade-delete issues or
  documents.
- Stage 4 repositories can attach to projects without rewriting planning tables.

## Rejected alternatives

- **Denormalized progress columns** — stale after every issue mutation.
- **Average of project percentages for initiatives** — overweights small projects.
- **Issue columns `cycle_id`/`milestone_id`** — touched all issue summary/detail
  SQL and filters before the planning UX was validated; join rows are cheaper now.
- **Storing cycle status** — requires a scheduler to stay correct offline.
- **Rich-text document blocks** — conflicts with Markdown portability and the spec.
- **Seeding repositories/PR counts** — would be fake data before Stage 4.
