# Stage 10 — repository model correction

**Date:** 2026-11-16
**Schema:** v9 → v10
**Status:** implemented on feature branch; CI verified; not merged

## Scope

- Normalized logical `repository` identity and remote-only repositories.
- One optional `repository_checkout` per repository; attach/detach without deleting files. Multiple clones per remote are explicitly out of scope for v1.
- `repository_project` many-to-many links, workspace-checked.
- Non-destructive v10 migration preserving legacy repository IDs, paths, project
  links, Issues and remote links.
- GitHub discovery upsert by provider identity with visible pagination state.
- GitHub import split into network fetch and short atomic DB apply; repository and
  project association are created before Issues; explicit target when multiple
  projects are linked.
- Repository and project UI: unified list, remote-only detail, project Repositories
  tab, link/unlink, checkout attachment.
- PR selected repository persisted by repository ID in SQLite settings.

## Verification

- `cargo test -p forgedesk-core` — 106 passed, 2 ignored.
- `npm test` — 139 passed.
- Visual gate — 59 captures.
- Feature branch `fix/repository-project-associations`, final code commit `bd771b6`.
- GitHub Actions run `43958233990`: all seven jobs passed.
- Not merged to main.

## Remaining risks

- No live GitHub network test in this pass; HTTP contracts remain mock/fixture based.
- One local checkout per logical repository is an explicit v1 limitation.
- AppImage/native packaging is not part of this correction.
- Existing provider-specific sync write limitations remain.
