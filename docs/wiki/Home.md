# ForgeDesk Wiki

ForgeDesk is a local-first desktop workspace for issues, projects, cycles,
documents and provider-backed development workflows. It runs on Windows and Linux
with Rust (Tauri 2), Vue 3 + TypeScript and SQLite.

## Start here

- [Architecture](Architecture)
- [Development](Development)
- [Project stages](Project-Stages)
- [Testing and release](Testing-and-Release)
- [Security and privacy](Security-and-Privacy)

## Current state

- Stage 1 — desktop foundation: complete (browser-verified).
- Stage 2 — issues/workflow/search/views: complete (browser-verified).
- Stage 3 — projects/planning/documents: partial; templates, document import and
  cycle analytics are follow-ups.
- Stage 4 — Local Git and API budget foundation: partial; GitHub App auth, live
  provider client and dispatcher remain open.
- Stage 5 — GitHub issue synchronization: blocked on GitHub App authentication.

The source of truth for project status is `docs/PROJECT_STATE.md` in the repository.
