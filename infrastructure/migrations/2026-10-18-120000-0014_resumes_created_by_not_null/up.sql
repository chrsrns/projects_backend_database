-- Pre-deploy check: SELECT count(*) FROM resumes WHERE created_by IS NULL;
-- A NULL row aborts this migration — fix the row first; the startup abort is deliberate.
ALTER TABLE resumes ALTER COLUMN created_by SET NOT NULL;
