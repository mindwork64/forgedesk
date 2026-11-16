# ForgeDesk release checklist

## Automated gates

- [x] `cargo fmt --all --check`
- [x] `cargo clippy -p forgedesk-core --all-targets -- -D warnings` (local)
- [x] `cargo clippy -p forgedesk-ai --all-targets -- -D warnings` (local)
- [x] AI Clippy command configured in the CI job with explicit `components: clippy`
- [x] AI Clippy CI execution verified successfully (run 43772258572)
- [x] `cargo test -p forgedesk-core` (104 passed, 2 ignored live)
- [x] `cargo test -p forgedesk-ai` (5 passed)
- [x] `npm run typecheck`
- [x] `npm test` (136 passed)
- [x] `npm run build` (main chunk ~244 KB)
- [x] `VITE_AI_EXTENSION=true npm run build`
- [x] `node scripts/visual-gate0.mjs` (59 captures + axe button-name/aria-command-name)
- [x] Realistic v5→v10 migration data-preservation test + foreign_key_check

## Verified CI / native build (non-interactive)

- [x] GitHub Actions CI [43778341394](https://github.com/mindwork64/forgedesk/actions/runs/43778341394)
      at `bddc4c58` — SUCCESS, all seven jobs
- [x] Windows 10 Pro Build 19042 native run
      [43778393711](https://github.com/mindwork64/forgedesk/actions/runs/43778393711)
      at `bddc4c58` — SUCCESS
- [x] Windows frontend typecheck/tests/build
- [x] Windows `cargo fmt --all --check`
- [x] Windows `cargo clippy -p forgedesk-core --all-targets -- -D warnings`
- [x] Windows `cargo test -p forgedesk-core` (Core only)
- [x] Windows `cargo build -p forgedesk` — Debug `forgedesk.exe`, 7,968,310 bytes
      ([artifact](https://github.com/mindwork64/forgedesk/actions/runs/43778393711/artifacts/14187497885))
- [x] Windows `cargo check -p forgedesk --features ai-extension` (check only)
- [ ] MSI/release installer and interactive Windows launch — not verified

## Native interactive smoke tests (not yet run)

- [ ] Windows 10/11 installer launch, WebView2 present
- [ ] Linux AppImage/deb launch with WebKitGTK 4.1
- [ ] Create workspace, issue, project, document
- [ ] Backup/restore round-trip
- [ ] Local Git repository attach/status
- [ ] GitHub Device Flow connect and read-only PR sync
- [ ] Merge linked PR → issue Done
- [ ] AI extension build disabled by default; optional screen when enabled
- [ ] MCP STDIO binary explicit start and read-only tool

## Signing/release credentials

- [ ] Windows code-signing certificate
- [ ] Linux package/signing policy
- [ ] Version/CHANGELOG finalization
- [ ] Signed checksums and artifacts

## Known limitations

- Real-time GitHub webhooks are not implemented (on-demand PR refresh).
- GitLab/Gitea live hosts not verified.
- Real AI provider calls require user keys and are not tested in CI.
- Stage 5 local→remote issue writes/conflicts remain partial.
- No crash telemetry backend by design/offline policy.
- GitHub Actions emits non-fatal Node.js 20/runtime deprecation warnings; no dependency
  modernization was attempted.
- The temporary WIN10 self-hosted workflow was removed; its successful build evidence
  remains documented but it is not part of normal push/PR CI.
