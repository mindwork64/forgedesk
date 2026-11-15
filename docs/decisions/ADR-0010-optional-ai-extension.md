# ADR-0010 — Optional, approval-gated AI extension

**Date:** 2026-11-15
**Status:** accepted
**Stage:** 9

## Decisions

1. **Separate crate and Cargo feature.**
   - `forgedesk-ai` is an optional desktop dependency behind `ai-extension`.
   - Core never imports it; default builds contain no AI crate/UI nav.
2. **Off by default.**
   - Runtime `enabled` is false until explicitly configured.
   - No provider/key access occurs while disabled.
3. **Keyring for secrets, JSON for non-secret config.**
   - API key uses OS keyring; base URL/model/provider are non-secret JSON.
4. **Context must be selected and previewed.**
   - The UI sends only the selected issue fields; description inclusion is explicit.
5. **AI never mutates data directly.**
   - Suggestions are text. Applying to the description requires a confirmation dialog
     and the normal local update command. Remote writes are not available to AI.
6. **MCP is explicit and read-only.**
   - `forgedesk-mcp` is a STDIO binary started with context JSON.
   - Only `forgedesk_context` is exposed; no mutation tools.
7. **Provider support is OpenAI-compatible + Anthropic.**
   - HTTP contracts are mock-tested. Live provider verification requires user keys and
     is not performed in CI.

## Consequences

- Default app remains fully offline/local; AI is absent.
- Enabling AI is a build + runtime choice.
- Streaming, model discovery, token accounting and MCP writes remain future work.
