CREATE TABLE pull_request_link (
    id             TEXT PRIMARY KEY NOT NULL,
    local_issue_id TEXT NOT NULL REFERENCES issue(id) ON DELETE CASCADE,
    provider       TEXT NOT NULL,
    account_id     TEXT NOT NULL,
    repository_id  TEXT NOT NULL,
    pull_number    INTEGER NOT NULL,
    pull_url       TEXT NOT NULL,
    title          TEXT NOT NULL,
    state          TEXT NOT NULL,
    created_at     TEXT NOT NULL,
    updated_at     TEXT NOT NULL,
    UNIQUE (provider, account_id, repository_id, pull_number)
) STRICT;

CREATE INDEX pull_request_link_issue ON pull_request_link(local_issue_id);
