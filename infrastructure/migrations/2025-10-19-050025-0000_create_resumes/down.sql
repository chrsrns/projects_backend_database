-- This file should undo anything in `up.sql`
-- Drop tables in reverse order (child tables first due to foreign key constraints)

DROP TABLE IF EXISTS portfolio_technologies;
DROP TABLE IF EXISTS portfolio_key_points;
DROP TABLE IF EXISTS portfolio_projects;
DROP TABLE IF EXISTS work_experience_key_points;
DROP TABLE IF EXISTS work_experiences;
DROP TABLE IF EXISTS education_key_points;
DROP TABLE IF EXISTS education;
DROP TABLE IF EXISTS frameworks;
DROP TABLE IF EXISTS languages;
DROP TABLE IF EXISTS skills;
DROP TABLE IF EXISTS resumes;
