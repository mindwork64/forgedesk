CREATE TABLE inbox_state (
    event_id    TEXT PRIMARY KEY NOT NULL REFERENCES activity_event(id) ON DELETE CASCADE,
    read_at     TEXT,
    archived_at TEXT
) STRICT;
