CREATE TABLE dashboard (
    id           TEXT PRIMARY KEY NOT NULL,
    workspace_id TEXT NOT NULL UNIQUE REFERENCES workspace(id) ON DELETE CASCADE,
    name         TEXT NOT NULL,
    created_at   TEXT NOT NULL,
    updated_at   TEXT NOT NULL
) STRICT;

CREATE TABLE dashboard_widget (
    id           TEXT PRIMARY KEY NOT NULL,
    dashboard_id TEXT NOT NULL REFERENCES dashboard(id) ON DELETE CASCADE,
    kind         TEXT NOT NULL CHECK (kind IN ('metric_open', 'metric_completed', 'metric_overdue', 'issues_recent', 'api_budget')),
    title        TEXT NOT NULL,
    width        INTEGER NOT NULL CHECK (width IN (3, 4, 6, 8, 12)),
    position     INTEGER NOT NULL,
    created_at   TEXT NOT NULL,
    updated_at   TEXT NOT NULL
) STRICT;

CREATE INDEX dashboard_widget_position ON dashboard_widget(dashboard_id, position);
