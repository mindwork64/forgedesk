-- Stage 11: per-project issue-key prefix overrides.
-- NULL means the project inherits the workspace-wide prefix.

ALTER TABLE project ADD COLUMN issue_key_prefix TEXT
    CHECK (
        issue_key_prefix IS NULL OR (
            issue_key_prefix = trim(issue_key_prefix)
            AND length(issue_key_prefix) BETWEEN 1 AND 10
            AND substr(issue_key_prefix, 1, 1) GLOB '[A-Z]'
            AND issue_key_prefix NOT GLOB '*[^A-Z0-9]*'
        )
    );
