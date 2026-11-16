-- Stage 10: logical repositories, local checkouts and project many-to-many links.

ALTER TABLE repository RENAME TO repository_legacy;

CREATE TABLE repository (
    id                   TEXT PRIMARY KEY NOT NULL,
    workspace_id         TEXT NOT NULL REFERENCES workspace(id) ON DELETE CASCADE,
    provider             TEXT NOT NULL CHECK (provider IN ('local', 'github', 'gitlab', 'gitea')),
    host                 TEXT NOT NULL,
    external_id          TEXT,
    owner                TEXT,
    name                 TEXT NOT NULL CHECK (length(trim(name)) BETWEEN 1 AND 200),
    full_name            TEXT NOT NULL CHECK (length(trim(full_name)) BETWEEN 1 AND 300),
    description          TEXT,
    visibility           TEXT CHECK (visibility IN ('public', 'private', 'internal')),
    default_branch       TEXT,
    html_url             TEXT,
    clone_url            TEXT,
    canonical_remote_url TEXT,
    is_archived          INTEGER NOT NULL DEFAULT 0 CHECK (is_archived IN (0, 1)),
    created_at           TEXT NOT NULL,
    updated_at           TEXT NOT NULL
) STRICT;

INSERT INTO repository
    (id, workspace_id, provider, host, external_id, owner, name, full_name,
     description, visibility, default_branch, html_url, clone_url,
     canonical_remote_url, is_archived, created_at, updated_at)
SELECT id, workspace_id, provider,
       CASE provider WHEN 'local' THEN 'local' ELSE '' END,
       NULL, NULL, name, name, NULL, NULL, default_branch, NULL, remote_url,
       NULL, 0, created_at, updated_at
FROM repository_legacy;

CREATE TABLE repository_checkout (
    id              TEXT PRIMARY KEY NOT NULL,
    workspace_id    TEXT NOT NULL REFERENCES workspace(id) ON DELETE CASCADE,
    repository_id   TEXT NOT NULL REFERENCES repository(id) ON DELETE CASCADE,
    path            TEXT NOT NULL,
    last_checked_at TEXT,
    created_at      TEXT NOT NULL,
    updated_at      TEXT NOT NULL,
    UNIQUE (workspace_id, path)
) STRICT;

INSERT INTO repository_checkout
    (id, workspace_id, repository_id, path, last_checked_at, created_at, updated_at)
SELECT 'checkout-' || id, workspace_id, id, path, last_checked_at, created_at, updated_at
FROM repository_legacy
WHERE length(trim(path)) > 0;

CREATE TABLE repository_project (
    workspace_id  TEXT NOT NULL REFERENCES workspace(id) ON DELETE CASCADE,
    repository_id TEXT NOT NULL REFERENCES repository(id) ON DELETE CASCADE,
    project_id    TEXT NOT NULL REFERENCES project(id) ON DELETE CASCADE,
    created_at    TEXT NOT NULL,
    PRIMARY KEY (workspace_id, repository_id, project_id)
) STRICT;

INSERT INTO repository_project (workspace_id, repository_id, project_id, created_at)
SELECT workspace_id, id, project_id, updated_at
FROM repository_legacy
WHERE project_id IS NOT NULL;

CREATE TRIGGER repository_project_same_workspace_insert
BEFORE INSERT ON repository_project
BEGIN
    SELECT CASE
        WHEN NOT EXISTS (
            SELECT 1 FROM repository r
            JOIN project p ON p.id = NEW.project_id
            WHERE r.id = NEW.repository_id
              AND r.workspace_id = NEW.workspace_id
              AND p.workspace_id = NEW.workspace_id
        )
        THEN RAISE(ABORT, 'repository and project must belong to the same workspace')
    END;
END;

DROP TABLE repository_legacy;

CREATE UNIQUE INDEX repository_external_identity
    ON repository(workspace_id, provider, host, external_id)
    WHERE external_id IS NOT NULL;

CREATE UNIQUE INDEX repository_canonical_remote
    ON repository(workspace_id, canonical_remote_url)
    WHERE canonical_remote_url IS NOT NULL;

CREATE INDEX repository_workspace ON repository(workspace_id, provider);
CREATE INDEX repository_checkout_repository ON repository_checkout(repository_id);
CREATE INDEX repository_project_project ON repository_project(project_id);

ALTER TABLE remote_issue_link RENAME COLUMN repository_id TO remote_repository_key;
