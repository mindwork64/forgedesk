# ForgeDesk release checklist

## Automated gates

- [x] `cargo fmt --all --check`
- [x] `cargo clippy -p forgedesk-core -p forgedesk-ai --all-targets -- -D warnings`
- [x] `cargo test -p forgedesk-core` (91 passed, 3 ignored live)
- [x] `cargo test -p forgedesk-ai` (5 passed)
- [x] `npm run typecheck`
- [x] `npm test` (107 passed)
- [x] `npm run build` (main chunk ~231 KB)
- [x] `VITE_AI_EXTENSION=true npm run build`
- [x] `node scripts/visual-gate0.mjs` (57 captures + accessibility names)
- [x] Migration v5→v9 test

## Native smoke tests (blocked here)

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
