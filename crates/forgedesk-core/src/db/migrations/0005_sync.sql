-- Stage 5 foundation: remote links, sync cursors, outbox and conflicts.

CREATE TABLE remote_issue_link (
    id                TEXT PRIMARY KEY NOT NULL,
    local_issue_id    TEXT NOT NULL UNIQUE REFERENCES issue(id) ON DELETE CASCADE,
    provider          TEXT NOT NULL CHECK (provider IN ('github', 'gitlab', 'gitea')),
    account_id        TEXT NOT NULL,
    repository_id     TEXT,
    remote_id         TEXT NOT NULL,
    remote_number     INTEGER NOT NULL,
    remote_url        TEXT NOT NULL,
    remote_updated_at TEXT,
    last_synced_at    TEXT,
    field_ownership   TEXT NOT NULL DEFAULT '{}',
    state             TEXT NOT NULL DEFAULT 'linked' CHECK (state IN ('linked', 'unlinked')),
    created_at        TEXT NOT NULL,
    updated_at        TEXT NOT NULL,
    UNIQUE (provider, account_id, remote_id)
) STRICT;

CREATE UNIQUE INDEX remote_issue_number
    ON remote_issue_link(provider, account_id, repository_id, remote_number);

CREATE TABLE sync_cursor (
    id                TEXT PRIMARY KEY NOT NULL,
    provider          TEXT NOT NULL,
    account_id        TEXT NOT NULL,
    resource          TEXT NOT NULL,
    scope             TEXT NOT NULL,
    page_cursor       TEXT,
    watermark         TEXT,
    last_success_at   TEXT,
    last_attempt_at   TEXT,
    next_due_at       TEXT,
    last_error        TEXT,
    incomplete_reason TEXT,
    updated_at        TEXT NOT NULL,
    UNIQUE (provider, account_id, resource, scope)
) STRICT;

CREATE TABLE sync_outbox (
    id                TEXT PRIMARY KEY NOT NULL,
    local_issue_id    TEXT REFERENCES issue(id) ON DELETE CASCADE,
    operation         TEXT NOT NULL,
    payload_json      TEXT NOT NULL DEFAULT '{}',
    state             TEXT NOT NULL DEFAULT 'pending' CHECK (
        state IN ('pending', 'dispatching', 'succeeded', 'failed_definite',
                  'outcome_unknown', 'reconciling', 'retry_safe',
                  'needs_user_action', 'cancelled')
    ),
    attempts          INTEGER NOT NULL DEFAULT 0,
    idempotency_key   TEXT NOT NULL UNIQUE,
    fingerprint       TEXT NOT NULL,
    reconciliation_hint TEXT,
    last_error        TEXT,
    created_at        TEXT NOT NULL,
    updated_at        TEXT NOT NULL
) STRICT;

CREATE INDEX sync_outbox_state ON sync_outbox(state, created_at);

CREATE TABLE sync_conflict (
    id                TEXT PRIMARY KEY NOT NULL,
    local_issue_id    TEXT NOT NULL REFERENCES issue(id) ON DELETE CASCADE,
    field             TEXT NOT NULL,
    local_value       TEXT,
    remote_value      TEXT,
    remote_updated_at TEXT,
    resolution        TEXT,
    resolved_at       TEXT,
    created_at        TEXT NOT NULL
) STRICT;

CREATE INDEX sync_conflict_issue ON sync_conflict(local_issue_id, resolved_at);
