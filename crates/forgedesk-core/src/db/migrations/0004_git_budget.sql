-- Stage 4: local repositories, provider accounts, API budget and request log.

CREATE TABLE repository (
    id              TEXT PRIMARY KEY NOT NULL,
    workspace_id    TEXT NOT NULL REFERENCES workspace(id) ON DELETE CASCADE,
    name            TEXT NOT NULL CHECK (length(trim(name)) BETWEEN 1 AND 160),
    path            TEXT NOT NULL,
    provider        TEXT NOT NULL DEFAULT 'local' CHECK (
        provider IN ('local', 'github', 'gitlab', 'gitea')
    ),
    remote_url      TEXT,
    default_branch  TEXT,
    project_id      TEXT REFERENCES project(id) ON DELETE SET NULL,
    last_checked_at TEXT,
    created_at      TEXT NOT NULL,
    updated_at      TEXT NOT NULL,
    UNIQUE (workspace_id, path)
) STRICT;

CREATE INDEX repository_workspace ON repository(workspace_id, provider);

CREATE TABLE provider_account (
    id             TEXT PRIMARY KEY NOT NULL,
    workspace_id   TEXT NOT NULL REFERENCES workspace(id) ON DELETE CASCADE,
    provider       TEXT NOT NULL CHECK (provider IN ('github', 'gitlab', 'gitea')),
    host           TEXT NOT NULL,
    account_login  TEXT NOT NULL,
    auth_kind      TEXT NOT NULL CHECK (auth_kind IN ('github_app', 'oauth', 'token')),
    scopes         TEXT NOT NULL DEFAULT '[]',
    status         TEXT NOT NULL DEFAULT 'connected' CHECK (
        status IN ('connected', 'expired', 'revoked', 'error')
    ),
    connected_at   TEXT NOT NULL,
    last_sync_at   TEXT,
    last_error     TEXT,
    UNIQUE (workspace_id, provider, host, account_login)
) STRICT;

CREATE TABLE api_budget_observation (
    id                   TEXT PRIMARY KEY NOT NULL,
    workspace_id         TEXT NOT NULL REFERENCES workspace(id) ON DELETE CASCADE,
    provider_account_id  TEXT,
    api_family           TEXT NOT NULL,
    resource_name        TEXT NOT NULL,
    limit_value          INTEGER,
    remaining_value      INTEGER,
    used_value           INTEGER,
    reset_at_utc         TEXT,
    observed_at_utc      TEXT NOT NULL,
    http_status          INTEGER,
    source               TEXT NOT NULL CHECK (
        source IN ('response_headers', 'graphql_rate_limit', 'manual_probe')
    ),
    is_estimate          INTEGER NOT NULL DEFAULT 0 CHECK (is_estimate IN (0, 1)),
    secondary_pause_until TEXT,
    notes                TEXT NOT NULL DEFAULT ''
) STRICT;

CREATE INDEX api_budget_workspace ON api_budget_observation(workspace_id, observed_at_utc DESC);

CREATE TABLE api_request_log (
    id                   TEXT PRIMARY KEY NOT NULL,
    workspace_id         TEXT NOT NULL REFERENCES workspace(id) ON DELETE CASCADE,
    provider_account_id  TEXT,
    operation_name       TEXT NOT NULL,
    operation_code       TEXT NOT NULL,
    api_family           TEXT NOT NULL,
    resource_name        TEXT NOT NULL,
    request_class        TEXT NOT NULL CHECK (
        request_class IN ('interactive_read', 'interactive_write', 'foreground_refresh',
                          'background_sync', 'manual_full_sync', 'reconciliation')
    ),
    fingerprint          TEXT NOT NULL,
    estimated_cost       REAL NOT NULL DEFAULT 0,
    actual_cost          REAL NOT NULL DEFAULT 0,
    http_status          INTEGER,
    error_category       TEXT,
    latency_ms           INTEGER,
    received_at_utc      TEXT NOT NULL,
    retry_attempt        INTEGER NOT NULL DEFAULT 0,
    result_state         TEXT NOT NULL CHECK (
        result_state IN ('success', 'not_modified', 'failed', 'rate_limited',
                         'secondary_limited', 'outcome_unknown', 'cancelled')
    ),
    rate_headers_json    TEXT NOT NULL DEFAULT '{}',
    payload_bytes        INTEGER,
    CHECK (length(fingerprint) <= 256)
) STRICT;

CREATE INDEX api_request_workspace ON api_request_log(workspace_id, received_at_utc DESC);

CREATE TABLE api_queue_job (
    id                TEXT PRIMARY KEY NOT NULL,
    workspace_id      TEXT NOT NULL REFERENCES workspace(id) ON DELETE CASCADE,
    provider_account_id TEXT,
    operation_name    TEXT NOT NULL,
    operation_code    TEXT NOT NULL,
    api_family        TEXT NOT NULL,
    priority          INTEGER NOT NULL CHECK (priority BETWEEN 0 AND 4),
    request_class     TEXT NOT NULL,
    state             TEXT NOT NULL DEFAULT 'queued' CHECK (
        state IN ('queued', 'paused', 'running', 'failed', 'outcome_unknown', 'cancelled', 'done')
    ),
    attempts          INTEGER NOT NULL DEFAULT 0,
    estimated_cost    REAL NOT NULL DEFAULT 0,
    fingerprint       TEXT NOT NULL,
    created_at        TEXT NOT NULL,
    updated_at        TEXT NOT NULL,
    last_error        TEXT
) STRICT;

CREATE INDEX api_queue_workspace ON api_queue_job(workspace_id, state, priority);

CREATE TABLE api_control (
    workspace_id TEXT PRIMARY KEY NOT NULL REFERENCES workspace(id) ON DELETE CASCADE,
    paused       INTEGER NOT NULL DEFAULT 0 CHECK (paused IN (0, 1)),
    reason       TEXT NOT NULL DEFAULT '',
    updated_at   TEXT NOT NULL
) STRICT;
