ALTER TABLE resumes
ADD COLUMN is_public BOOLEAN NOT NULL DEFAULT FALSE;

CREATE INDEX idx_resumes_is_public ON resumes(is_public);
