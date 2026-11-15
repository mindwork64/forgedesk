# Security and privacy

- Local-first: SQLite data stays on the machine; no mandatory backend.
- No Authorization headers, tokens, private issue bodies, diffs or comments in the
  API request log.
- Provider credentials must never be stored in SQLite or committed. Stage 4 keeps
  provider account identity/scopes only; GitHub Connect stays disabled until an OS
  credential vault + GitHub App flow exist.
- Git operations use direct argv `Command` calls (`git`), never a shell; user values
  are not interpolated into a command string. Branch create/switch requires a clean
  worktree.
- API budget background allowance reserves 20% of the observed limit and caps
  background use at 10%; unknown values yield zero background allowance.
- Backup snapshots are local; secrets are not included by design.
- Do not put Jenkins/GitHub tokens in the repository, docs, CI logs or screenshots.
