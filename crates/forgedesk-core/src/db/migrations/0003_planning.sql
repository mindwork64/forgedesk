-- Stage 3: initiatives, project metadata, milestones, cycles and documents.

CREATE TABLE initiative (
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

CREATE INDEX initiative_workspace ON initiative(workspace_id, archived_at);

ALTER TABLE project ADD COLUMN initiative_id TEXT REFERENCES initiative(id) ON DELETE SET NULL;
ALTER TABLE project ADD COLUMN color TEXT NOT NULL DEFAULT 'accent';
ALTER TABLE project ADD COLUMN progress_method TEXT NOT NULL DEFAULT 'issues';

CREATE INDEX project_initiative ON project(initiative_id);

CREATE TABLE milestone (
    id          TEXT PRIMARY KEY NOT NULL,
    project_id  TEXT NOT NULL REFERENCES project(id) ON DELETE CASCADE,
    name        TEXT NOT NULL CHECK (length(trim(name)) BETWEEN 1 AND 120),
    description TEXT NOT NULL DEFAULT '',
    due_date    TEXT,
    status      TEXT NOT NULL DEFAULT 'planned' CHECK (
        status IN ('planned', 'active', 'completed', 'canceled')
    ),
    archived_at TEXT,
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL
) STRICT;

CREATE INDEX milestone_project ON milestone(project_id, archived_at);

CREATE TABLE cycle (
    id           TEXT PRIMARY KEY NOT NULL,
    workspace_id TEXT NOT NULL REFERENCES workspace(id) ON DELETE CASCADE,
    name         TEXT NOT NULL CHECK (length(trim(name)) BETWEEN 1 AND 120),
    start_date   TEXT NOT NULL,
    end_date     TEXT NOT NULL,
    archived_at  TEXT,
    created_at   TEXT NOT NULL,
    updated_at   TEXT NOT NULL,
    CHECK (start_date <= end_date)
) STRICT;

CREATE INDEX cycle_workspace ON cycle(workspace_id, start_date DESC);

CREATE TABLE document (
    id            TEXT PRIMARY KEY NOT NULL,
    workspace_id  TEXT NOT NULL REFERENCES workspace(id) ON DELETE CASCADE,
    title         TEXT NOT NULL CHECK (length(trim(title)) BETWEEN 1 AND 200),
    body          TEXT NOT NULL DEFAULT '',
    project_id    TEXT REFERENCES project(id) ON DELETE SET NULL,
    initiative_id TEXT REFERENCES initiative(id) ON DELETE SET NULL,
    archived_at   TEXT,
    created_at    TEXT NOT NULL,
    updated_at    TEXT NOT NULL
) STRICT;

CREATE INDEX document_workspace ON document(workspace_id, updated_at DESC);
CREATE INDEX document_project ON document(project_id);

CREATE TABLE issue_cycle (
    issue_id  TEXT PRIMARY KEY NOT NULL REFERENCES issue(id) ON DELETE CASCADE,
    cycle_id  TEXT NOT NULL REFERENCES cycle(id) ON DELETE CASCADE,
    added_at  TEXT NOT NULL
) STRICT;

CREATE INDEX issue_cycle_cycle ON issue_cycle(cycle_id);

CREATE TABLE issue_milestone (
    issue_id     TEXT PRIMARY KEY NOT NULL REFERENCES issue(id) ON DELETE CASCADE,
    milestone_id TEXT NOT NULL REFERENCES milestone(id) ON DELETE CASCADE,
    added_at     TEXT NOT NULL
) STRICT;

CREATE INDEX issue_milestone_milestone ON issue_milestone(milestone_id);
