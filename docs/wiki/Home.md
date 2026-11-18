# ForgeDesk Wiki

ForgeDesk is a local-first, single-user desktop workspace for issues, planning, Git and provider-backed development workflows. It runs on Windows and Linux with Rust (Tauri 2), Vue 3/TypeScript and SQLite.

## Start here

- [Architecture](Architecture)
- [Development](Development)
- [Project stages](Project-Stages)
- [Full product roadmap](Product-Roadmap)
- [API Budget and Sync](API-Budget-and-Sync)
- [Testing and release](Testing-and-Release)
- [Security and privacy](Security-and-Privacy)

## Current state — 2026-11-18

Core local functionality and major GitHub screens exist. PR #12 added shared GitHub REST dispatch and enforced REST pause/resume. ForgeDesk is **not complete**: Issues/Views and Planning/Documents have gaps; Local Git is a safe subset; GitHub GraphQL/budget scheduling/automatic incremental sync and complete Issue comments are unfinished; Inbox/Triage/Automations/Insights, GitLab/Gitea and AI have remaining scope; native runtime and release hardening remain open.

The full backlog, acceptance matrix and DeepSeek starter prompt are in docs/specs/PRODUCT_COMPLETION_SPEC.md. docs/PROJECT_STATE.md is the compact implementation truth.
