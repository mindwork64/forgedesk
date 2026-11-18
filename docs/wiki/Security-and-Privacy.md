# Security and privacy

- Local-first: SQLite data stays on the machine; no mandatory backend.
- Provider credentials must never be stored in SQLite, scheduler payloads, API logs
  or committed. GitHub tokens live in the OS keyring; if unavailable, session-only
  memory fallback is used with a UI warning.
- OAuth Device Flow/token exchange/refresh are separate auth operations. Authenticated
  GitHub data API requests use the shared dispatcher; the authenticated identity
  `GET /user` goes through the REST client.
- API request fingerprints must not include access tokens. Never log Authorization
  headers, raw Issue body, diff or comments.
- Background quota policy uses a 10% allowance cap and 20% safety reserve. Unknown
  or stale budget observations yield zero background allowance.
- A user pause blocks new authenticated GitHub data requests and must not discard
  queued work or corrupt Outbox state.
- Mutations with ambiguous network outcomes are reconciled before retry; never
  blindly replay a potentially applied remote write.
- Git operations use direct argv `Command` calls, never a shell; user values are not
  interpolated into a command string.
- Backup snapshots are local; secrets are not included by design.
