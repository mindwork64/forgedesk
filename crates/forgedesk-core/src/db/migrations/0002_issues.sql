-- Stage 2: issues, workflow, labels, relations, comments, activity and search.

CREATE TABLE workflow (
    id           TEXT PRIMARY KEY NOT NULL,
    workspace_id TEXT NOT NULL REFERENCES workspace(id) ON DELETE CASCADE,
    name         TEXT NOT NULL CHECK (length(trim(name)) BETWEEN 1 AND 80),
    is_default   INTEGER NOT NULL DEFAULT 0 CHECK (is_default IN (0, 1)),
    created_at   TEXT NOT NULL,
    UNIQUE (workspace_id, name)
) STRICT;

CREATE UNIQUE INDEX workflow_one_default_per_workspace
    ON workflow(workspace_id) WHERE is_default = 1;

CREATE TABLE workflow_status (
    id          TEXT PRIMARY KEY NOT NULL,
    workflow_id TEXT NOT NULL REFERENCES workflow(id) ON DELETE CASCADE,
    name        TEXT NOT NULL CHECK (length(trim(name)) BETWEEN 1 AND 80),
    category    TEXT NOT NULL CHECK (
        category IN ('backlog', 'unstarted', 'started', 'completed', 'canceled')
    ),
    color       TEXT NOT NULL CHECK (
        color IN ('neutral', 'info', 'accent', 'success', 'warning', 'danger')
    ),
    position    INTEGER NOT NULL CHECK (position >= 0),
    is_default  INTEGER NOT NULL DEFAULT 0 CHECK (is_default IN (0, 1)),
    system_key  TEXT,
    created_at  TEXT NOT NULL,
    UNIQUE (workflow_id, position),
    UNIQUE (workflow_id, system_key)
) STRICT;

CREATE UNIQUE INDEX workflow_status_one_default
    ON workflow_status(workflow_id) WHERE is_default = 1;

CREATE TABLE project (
    id           TEXT PRIMARY KEY NOT NULL,
    workspace_id TEXT NOT NULL REFERENCES workspace(id) ON DELETE CASCADE,
    name         TEXT NOT NULL CHECK (length(trim(name)) BETWEEN 1 AND 120),
    description  TEXT NOT NULL DEFAULT '',
    status       TEXT NOT NULL DEFAULT 'planned' CHECK (
        status IN ('planned', 'active', 'paused', 'completed', 'canceled')
    ),
    priority     TEXT NOT NULL DEFAULT 'none' CHECK (
        priority IN ('none', 'low', 'medium', 'high', 'urgent')
    ),
    start_date   TEXT,
    target_date  TEXT,
    archived_at  TEXT,
    created_at   TEXT NOT NULL,
    updated_at   TEXT NOT NULL
) STRICT;

CREATE INDEX project_workspace ON project(workspace_id, archived_at);

CREATE TABLE label (
    id           TEXT PRIMARY KEY NOT NULL,
    workspace_id TEXT NOT NULL REFERENCES workspace(id) ON DELETE CASCADE,
    name         TEXT NOT NULL CHECK (length(trim(name)) BETWEEN 1 AND 40),
    color        TEXT NOT NULL DEFAULT 'neutral' CHECK (
        color IN ('neutral', 'info', 'accent', 'success', 'warning', 'danger')
    ),
    created_at   TEXT NOT NULL
) STRICT;

CREATE UNIQUE INDEX label_name_per_workspace
    ON label(workspace_id, lower(name));

CREATE TABLE workspace_key_counter (
    workspace_id TEXT PRIMARY KEY NOT NULL REFERENCES workspace(id) ON DELETE CASCADE,
    next_number  INTEGER NOT NULL CHECK (next_number >= 1)
) STRICT;

CREATE TABLE issue (
    id           TEXT PRIMARY KEY NOT NULL,
    workspace_id TEXT NOT NULL REFERENCES workspace(id) ON DELETE CASCADE,
    key_number   INTEGER NOT NULL CHECK (key_number >= 1),
    key          TEXT NOT NULL,
    title        TEXT NOT NULL CHECK (length(trim(title)) BETWEEN 1 AND 500),
    description  TEXT NOT NULL DEFAULT '',
    status_id    TEXT NOT NULL REFERENCES workflow_status(id),
    priority     TEXT NOT NULL DEFAULT 'none' CHECK (
        priority IN ('none', 'low', 'medium', 'high', 'urgent')
    ),
    issue_type   TEXT NOT NULL DEFAULT 'task' CHECK (
        issue_type IN ('bug', 'feature', 'task', 'improvement', 'chore', 'research')
    ),
    estimate     REAL CHECK (estimate IS NULL OR estimate >= 0),
    start_date   TEXT,
    due_date     TEXT,
    project_id   TEXT REFERENCES project(id) ON DELETE SET NULL,
    parent_id    TEXT REFERENCES issue(id) ON DELETE SET NULL,
    source       TEXT NOT NULL DEFAULT 'local' CHECK (
        source IN ('local', 'github', 'gitlab', 'gitea')
    ),
    remote_url   TEXT,
    archived_at  TEXT,
    deleted_at   TEXT,
    created_at   TEXT NOT NULL,
    updated_at   TEXT NOT NULL,
    completed_at TEXT,
    UNIQUE (workspace_id, key_number),
    UNIQUE (workspace_id, key)
) STRICT;

CREATE INDEX issue_workspace_status ON issue(workspace_id, status_id);
CREATE INDEX issue_workspace_updated ON issue(workspace_id, updated_at DESC);
CREATE INDEX issue_workspace_created ON issue(workspace_id, created_at DESC);
CREATE INDEX issue_parent ON issue(parent_id);
CREATE INDEX issue_project ON issue(project_id);
CREATE INDEX issue_archived ON issue(workspace_id, archived_at);

CREATE TABLE issue_label (
    issue_id TEXT NOT NULL REFERENCES issue(id) ON DELETE CASCADE,
    label_id TEXT NOT NULL REFERENCES label(id) ON DELETE CASCADE,
    PRIMARY KEY (issue_id, label_id)
) STRICT;

CREATE INDEX issue_label_label ON issue_label(label_id);

CREATE TABLE issue_relation (
    id            TEXT PRIMARY KEY NOT NULL,
    from_issue_id TEXT NOT NULL REFERENCES issue(id) ON DELETE CASCADE,
    to_issue_id   TEXT NOT NULL REFERENCES issue(id) ON DELETE CASCADE,
    relation_type TEXT NOT NULL CHECK (
        relation_type IN ('blocks', 'related', 'duplicate')
    ),
    created_at    TEXT NOT NULL,
    UNIQUE (from_issue_id, to_issue_id, relation_type),
    CHECK (from_issue_id <> to_issue_id)
) STRICT;

CREATE INDEX issue_relation_from ON issue_relation(from_issue_id);
CREATE INDEX issue_relation_to ON issue_relation(to_issue_id);

CREATE TABLE comment (
    id         TEXT PRIMARY KEY NOT NULL,
    issue_id   TEXT NOT NULL REFERENCES issue(id) ON DELETE CASCADE,
    body       TEXT NOT NULL CHECK (length(trim(body)) BETWEEN 1 AND 10000),
    author     TEXT NOT NULL DEFAULT 'local' CHECK (author IN ('local', 'remote')),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    deleted_at TEXT
) STRICT;

CREATE INDEX comment_issue ON comment(issue_id, created_at);

CREATE TABLE activity_event (
    id         TEXT PRIMARY KEY NOT NULL,
    issue_id   TEXT NOT NULL REFERENCES issue(id) ON DELETE CASCADE,
    kind       TEXT NOT NULL,
    data       TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL
) STRICT;

CREATE INDEX activity_issue ON activity_event(issue_id, created_at);

CREATE TABLE saved_view (
    id           TEXT PRIMARY KEY NOT NULL,
    workspace_id TEXT NOT NULL REFERENCES workspace(id) ON DELETE CASCADE,
    name         TEXT NOT NULL CHECK (length(trim(name)) BETWEEN 1 AND 80),
    entity       TEXT NOT NULL DEFAULT 'issue' CHECK (entity = 'issue'),
    filters      TEXT,
    sort         TEXT NOT NULL DEFAULT '[]',
    group_by     TEXT,
    columns      TEXT NOT NULL DEFAULT '[]',
    density      TEXT NOT NULL DEFAULT 'standard' CHECK (
        density IN ('compact', 'standard')
    ),
    is_pinned    INTEGER NOT NULL DEFAULT 0 CHECK (is_pinned IN (0, 1)),
    position     INTEGER NOT NULL DEFAULT 0,
    created_at   TEXT NOT NULL,
    updated_at   TEXT NOT NULL
) STRICT;

CREATE INDEX saved_view_workspace ON saved_view(workspace_id, position);

CREATE VIRTUAL TABLE issue_fts USING fts5(
    issue_id UNINDEXED,
    title,
    description,
    tokenize = 'unicode61 remove_diacritics 2'
);

CREATE TRIGGER issue_fts_insert AFTER INSERT ON issue BEGIN
    INSERT INTO issue_fts (issue_id, title, description)
    VALUES (new.id, new.title, new.description);
END;

CREATE TRIGGER issue_fts_delete AFTER DELETE ON issue BEGIN
    DELETE FROM issue_fts WHERE issue_id = old.id;
END;

CREATE TRIGGER issue_fts_update AFTER UPDATE OF title, description ON issue BEGIN
    DELETE FROM issue_fts WHERE issue_id = old.id;
    INSERT INTO issue_fts (issue_id, title, description)
    VALUES (new.id, new.title, new.description);
END;

-- Seed the default workflow, statuses and key counter for an existing workspace.
INSERT INTO workflow (id, workspace_id, name, is_default, created_at)
SELECT 'wf-' || w.id, w.id, 'Default', 1,
       strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
FROM workspace w;

INSERT INTO workflow_status
    (id, workflow_id, name, category, color, position, is_default, system_key, created_at)
SELECT 'st-' || w.workspace_id || '-backlog', w.id, 'Backlog',
       'backlog', 'neutral', 0, 1, 'backlog',
       strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
FROM workflow w;

INSERT INTO workflow_status
    (id, workflow_id, name, category, color, position, is_default, system_key, created_at)
SELECT 'st-' || w.workspace_id || '-todo', w.id, 'Todo',
       'unstarted', 'info', 1, 0, 'todo',
       strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
FROM workflow w;

INSERT INTO workflow_status
    (id, workflow_id, name, category, color, position, is_default, system_key, created_at)
SELECT 'st-' || w.workspace_id || '-in-progress', w.id, 'In Progress',
       'started', 'accent', 2, 0, 'in_progress',
       strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
FROM workflow w;

INSERT INTO workflow_status
    (id, workflow_id, name, category, color, position, is_default, system_key, created_at)
SELECT 'st-' || w.workspace_id || '-done', w.id, 'Done',
       'completed', 'success', 3, 0, 'done',
       strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
FROM workflow w;

INSERT INTO workflow_status
    (id, workflow_id, name, category, color, position, is_default, system_key, created_at)
SELECT 'st-' || w.workspace_id || '-canceled', w.id, 'Canceled',
       'canceled', 'neutral', 4, 0, 'canceled',
       strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
FROM workflow w;

INSERT INTO workspace_key_counter (workspace_id, next_number)
SELECT id, 1 FROM workspace;
