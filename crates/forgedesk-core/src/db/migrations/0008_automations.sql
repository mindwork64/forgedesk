CREATE TABLE automation_rule (
    id            TEXT PRIMARY KEY NOT NULL,
    workspace_id  TEXT NOT NULL REFERENCES workspace(id) ON DELETE CASCADE,
    name          TEXT NOT NULL CHECK (length(trim(name)) BETWEEN 1 AND 120),
    enabled       INTEGER NOT NULL DEFAULT 1 CHECK (enabled IN (0, 1)),
    trigger_event TEXT NOT NULL CHECK (trigger_event IN ('issue_created', 'status_changed')),
    action_type   TEXT NOT NULL CHECK (action_type = 'add_label'),
    action_value  TEXT NOT NULL,
    last_run_at   TEXT,
    last_result   TEXT,
    created_at    TEXT NOT NULL,
    updated_at    TEXT NOT NULL
) STRICT;

CREATE TABLE automation_run (
    id         TEXT PRIMARY KEY NOT NULL,
    rule_id    TEXT NOT NULL REFERENCES automation_rule(id) ON DELETE CASCADE,
    event_id   TEXT NOT NULL REFERENCES activity_event(id) ON DELETE CASCADE,
    status     TEXT NOT NULL,
    message    TEXT NOT NULL,
    created_at TEXT NOT NULL,
    UNIQUE (rule_id, event_id)
) STRICT;

CREATE INDEX automation_run_rule ON automation_run(rule_id, created_at DESC);
