# Stage 9 report — Optional AI extension

**Date:** 2026-11-15
**Status:** COMPLETE (fixtures) — the AI extension is implemented as a separate
optional crate and desktop feature, with OpenAI-compatible/Anthropic adapters, context
selection, approval-gated local apply and a read-only MCP STDIO bridge. No live AI
provider credentials were available, so provider calls were verified with local mock
HTTP servers only; this is stated in the UI.
**Schema:** v9 (AI settings live in a JSON file + OS keyring, not SQLite)

## Delivered

- Separate `crates/forgedesk-ai` crate:
  - core has no AI dependency;
  - desktop `ai-extension` Cargo feature is **off by default** and optional.
- Providers:
  - OpenAI-compatible `/chat/completions`;
  - Anthropic `/v1/messages`;
  - API key stored in OS keyring; runtime disabled by default; no key load when off.
- Context selection:
  - explicit issue key, status, project and optional description;
  - prompt/context preview shown before generation.
- Approval:
  - AI cannot mutate data directly;
  - “Append to description” requires a confirmation dialog and uses the normal local
    issue update command.
- MCP:
  - `forgedesk-mcp` STDIO binary, explicit `FORGEDESK_AI_CONTEXT` start;
  - read-only `forgedesk_context` tool; no mutation tools;
  - JSON-RPC initialize/ping/tools list/call.
- Build/UI optionality:
  - default desktop build does not link `forgedesk-ai`;
  - default frontend build has no AI nav/DOM;
  - `VITE_AI_EXTENSION=true` + `--features ai-extension` adds the AI Assistant screen.
- AI settings persisted as non-secret JSON; API key in OS keyring.

## Not delivered

- Live provider call verification (no provider key available).
- Provider-specific model discovery/streaming/token accounting.
- MCP write tools or automatic AI mutations.
- AI on Windows/Linux native runtime verification.

## Verification

| Check | Command | Result |
|---|---|---|
| Core tests | `cargo test -p forgedesk-core` | PASS — 90 (+3 ignored live) |
| AI crate tests | `cargo test -p forgedesk-ai` | PASS — 5 |
| Contract tests | mock OpenAI/Anthropic + MCP | PASS |
| Default desktop build | `cargo check -p forgedesk` | PASS (no forgedesk-ai dependency) |
| AI desktop build | `cargo check -p forgedesk --features ai-extension` | PASS |
| Frontend | `npm run typecheck`, `npm test` | PASS — 107 |
| Default visual gates | `node scripts/visual-gate0.mjs` | PASS — 57 captures, AI nav absent |
| AI frontend build | `VITE_AI_EXTENSION=true npm run build` | PASS |
| Optional AI capture | AI-enabled harness | PASS — `docs/ui/current/gate4-ai-extension-dark-1440.png` |
| Live AI provider | — | NOT RUN — no credentials |

## Next

Stage 10 stabilization/release; remaining Stage 5 sync writes and native verification
stay tracked.
