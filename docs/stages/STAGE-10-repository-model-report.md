# Stage 10 — repository model correction

**Date:** 2026-11-16
**Schema:** v9 → v10
**Status:** merged to `main` via PR #1; CI verified

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
- Squash merge commit on `main`: `670b499`.
- GitHub Actions run `43969772720`: all seven jobs passed.
- Windows Release Installer run `43969772719`: SUCCESS.

## Remaining risks

- No live GitHub network test in this pass; HTTP contracts remain mock/fixture based.
- One local checkout per logical repository is an explicit v1 limitation.
- AppImage/native packaging is not part of this correction.
- Existing provider-specific sync write limitations remain.
