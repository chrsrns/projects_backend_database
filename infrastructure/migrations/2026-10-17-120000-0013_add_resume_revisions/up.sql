-- Per-resume revision chain: one row per state changing edit, holding the
-- whole resume aggregate as it stood before that edit.
CREATE TABLE resume_revisions (
    resume_id UUID NOT NULL REFERENCES resumes(id) ON DELETE CASCADE,
    revision INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    created_by INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    action VARCHAR(32) NOT NULL,
    is_restore BOOLEAN NOT NULL DEFAULT FALSE,
    snapshot JSONB NOT NULL,
    PRIMARY KEY (resume_id, revision)
);

CREATE INDEX idx_resume_revisions_created_by ON resume_revisions (created_by);
