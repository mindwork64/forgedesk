# ForgeDesk release checklist — full product DoD

This checklist separates automated verification from native interactive and live-provider verification. Do not mark an item complete based only on compilation.

## Core/data integrity
- [x] Append-only SQLite migration runner and backup/restore baseline exist.
- [ ] Supported migration path includes v11 → current with all entities/links/outbox/cursors/attachments preserved.
- [ ] Workspace export/import includes DB + managed attachments, manifest/checksum and rollback.
- [ ] Offline startup/local CRUD verified on Windows and Linux.
- [ ] Large-workspace/performance/soak test completed.

## Local product
- [ ] Issues/Workflow/Views gaps in PRODUCT_COMPLETION_SPEC.md closed.
- [ ] Projects/Initiatives/Cycles/Documents/Roadmap gaps closed.
- [ ] Local Git safe operations/timeouts/cancellation/remote confirmations closed.
- [ ] UI Gates 0–5 pass in Dark/Light, RU/EN and supported viewports.
- [ ] Keyboard/accessibility audit passes; no inert controls or production fixtures.

## GitHub/providers
- [ ] REST + GraphQL share dispatcher/queue/budget; API Monitor shows truthful worker state.
- [ ] Automatic Outbox and incremental cursor sync pass restart/ambiguity tests.
- [ ] GitHub Issues/comments/labels/milestones and PR/review/CI end-to-end scenarios pass.
- [ ] GitLab/Gitea live capability matrix tested when hosts are available; otherwise NOT VERIFIED.
- [ ] AI default-off/build optionality/approval tests pass; live calls require user credentials.

## Automated CI
- [ ] cargo fmt --all --check
- [ ] Core tests + Clippy Ubuntu/Windows
- [ ] Optional AI tests/Clippy and feature check
- [ ] Frontend typecheck/tests/production build
- [ ] Tauri compile Linux/Windows
- [ ] Visual/accessibility gates and production harness exclusion
- [ ] GitHub-hosted Actions green on exact release candidate SHA

## Native interactive smoke
- [ ] Windows 10/11 WebView2 installer launch, update/uninstall and full workflow
- [ ] Linux WebKitGTK 4.1 launch/package install/uninstall and full workflow
- [ ] GitHub connect/import/sync/reconciliation and PR merge → local task completion
- [ ] Offline restart, backup/restore and export/import
- [ ] Local Git folder/terminal/IDE and guarded remote operations
- [ ] AI disabled by default; optional build behavior verified

## Packaging and documentation
- [ ] Version/CHANGELOG finalized, artifacts/checksums published.
- [ ] Windows MSI/installer and Linux package built.
- [ ] Signing credentials supplied before claiming a signed release.
- [ ] Installation/update/uninstall, data location, backup/restore, GitHub permissions and troubleshooting docs.
- [ ] Known limitations documented; no mandatory telemetry/backend.

See docs/specs/PRODUCT_COMPLETION_SPEC.md for complete acceptance and test matrix.
