# ADR-0007 — GitHub App Device Flow and token vault

**Date:** 2026-11-15  
**Status:** accepted  
**Stage:** 5 foundation

## Context

Stage 4 left GitHub Connect disabled because no app credentials/auth flow existed.
A public GitHub App Client ID is now available:
`Iv23liclPWLrC2OpaeXz`. The app still has no client secret and must not ship one.

## Decisions

1. **Use GitHub App Device Flow, not a shipped secret.**
   - The client ID is public and can be embedded; override with
     `FORGEDESK_GITHUB_CLIENT_ID` for other registrations.
   - No client secret/private key is stored or required by the desktop client.
2. **Backend owns the device flow.**
   - `start` requests a device code and keeps it in memory.
   - `poll` is called at the server interval, honors `authorization_pending` and
     `slow_down`, and surfaces expiry/denial as auth errors.
   - A device code is never persisted or logged.
3. **Token storage is OS keyring first.**
   - macOS Keychain, Windows Credential Manager, Linux Secret Service via `keyring`.
   - Key is `dev.forgedesk.app` / `github-app-user`; value is the token JSON.
   - If the keyring is unavailable, the token remains **memory-only** for the session
     and the UI displays a storage warning. There is no plaintext file fallback.
4. **Refresh before expiry.**
   - GitHub App user tokens are expiring (observed `expires_in=28800`) and include a
     refresh token. Refresh at ≤60 seconds before expiry, or explicitly from UI.
   - Refresh response rotates both access and refresh tokens.
   - Disconnect deletes the local keyring entry; remote revocation is a future
     explicit action.
5. **Tokens never enter SQLite, logs, diagnostics or screenshots.**
   - API Monitor diagnostics serialize budget/request/queue/control state only.
   - Auth errors carry messages, not token values.
6. **Live verification is required before sync.**
   - Manual test on 2026-11-15: device code issued, authorization completed,
     `/user` returned `mindwork64` (id 34349377), rate headers 5000/4996, refresh
     returned HTTP 200 with new access+refresh tokens and 28800s expiry.
   - No mass sync/write is enabled until the provider client and dispatcher are
     tested.

## Consequences

- Auth works on desktop with a keyring; headless CI can authenticate but may fall
  back to memory-only and lose the token on exit.
- Users may need to reauthorize if the refresh token expires/revokes.
- Stage 5 sync must obtain tokens through this service and never add token columns.
- The UI shows real connection state; screenshots with connected state are
  harness-gated or use the manual test account without persisting tokens.

## Rejected alternatives

- Shipping a client secret/private key in the desktop app — prohibited.
- Storing tokens in SQLite or a plaintext file — leaks via backups/copies.
- Long-lived PAT as the primary mechanism — conflicts with GitHub App requirement.
- Fake connected state before auth — violates no-fake-UI.
