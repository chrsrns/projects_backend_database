-- Drop triggers first to avoid dependency issues
DROP TRIGGER IF EXISTS update_users_modtime ON users;
DROP TRIGGER IF EXISTS update_sessions_modtime ON sessions;

-- Drop the function
DROP FUNCTION IF EXISTS update_modified_column();

-- Drop the created_by column from resumes
ALTER TABLE resumes DROP COLUMN IF EXISTS created_by;

-- Drop the sessions table
DROP TABLE IF EXISTS sessions;

-- Drop the users table
DROP TABLE IF EXISTS users;
