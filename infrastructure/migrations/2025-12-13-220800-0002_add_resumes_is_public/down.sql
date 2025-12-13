DROP INDEX IF EXISTS idx_resumes_is_public;

ALTER TABLE resumes
DROP COLUMN IF EXISTS is_public;
