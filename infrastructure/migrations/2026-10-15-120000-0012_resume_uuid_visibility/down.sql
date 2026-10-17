-- Reverse: UUID ids back to SERIAL ints (identity preserved via remap) and
-- visibility back to is_public (unlisted folds to private).

-- 1. is_public: 'public' -> true, 'private'/'unlisted' -> false
ALTER TABLE resumes ADD COLUMN is_public BOOLEAN;
UPDATE resumes SET is_public = (visibility = 'public');
ALTER TABLE resumes
    ALTER COLUMN is_public SET NOT NULL,
    ALTER COLUMN is_public SET DEFAULT FALSE;
DROP INDEX idx_resumes_visibility;
ALTER TABLE resumes DROP CONSTRAINT resumes_visibility_check;
ALTER TABLE resumes DROP COLUMN visibility;
CREATE INDEX idx_resumes_is_public ON resumes(is_public);

-- 2. new SERIAL identity on resumes
ALTER TABLE resumes ADD COLUMN old_id SERIAL;

-- 3. int self-FK column for variants
ALTER TABLE resumes ADD COLUMN old_base_resume_id INTEGER;
UPDATE resumes v SET old_base_resume_id = b.old_id
    FROM resumes b WHERE v.base_resume_id = b.id;

-- 4. int FK columns on children
ALTER TABLE skills ADD COLUMN old_resume_id INTEGER;
UPDATE skills c SET old_resume_id = r.old_id FROM resumes r WHERE c.resume_id = r.id;
ALTER TABLE languages ADD COLUMN old_resume_id INTEGER;
UPDATE languages c SET old_resume_id = r.old_id FROM resumes r WHERE c.resume_id = r.id;
ALTER TABLE education ADD COLUMN old_resume_id INTEGER;
UPDATE education c SET old_resume_id = r.old_id FROM resumes r WHERE c.resume_id = r.id;
ALTER TABLE work_experiences ADD COLUMN old_resume_id INTEGER;
UPDATE work_experiences c SET old_resume_id = r.old_id FROM resumes r WHERE c.resume_id = r.id;
ALTER TABLE portfolio_projects ADD COLUMN old_resume_id INTEGER;
UPDATE portfolio_projects c SET old_resume_id = r.old_id FROM resumes r WHERE c.resume_id = r.id;

-- 5. drop dependents on the uuid columns
DROP INDEX resumes_email_base_unique;
DROP INDEX idx_skills_resume_id;
DROP INDEX idx_languages_resume_id;
DROP INDEX idx_education_resume_id;
DROP INDEX idx_work_experiences_resume_id;
DROP INDEX idx_portfolio_projects_resume_id;
ALTER TABLE resumes DROP CONSTRAINT resumes_base_resume_id_fkey;
ALTER TABLE resumes DROP CONSTRAINT resumes_base_resume_id_not_self;
ALTER TABLE skills DROP CONSTRAINT skills_resume_id_fkey;
ALTER TABLE languages DROP CONSTRAINT languages_resume_id_fkey;
ALTER TABLE education DROP CONSTRAINT education_resume_id_fkey;
ALTER TABLE work_experiences DROP CONSTRAINT work_experiences_resume_id_fkey;
ALTER TABLE portfolio_projects DROP CONSTRAINT portfolio_projects_resume_id_fkey;
ALTER TABLE resumes DROP CONSTRAINT resumes_pkey;

-- 6. swap columns back
ALTER TABLE skills DROP COLUMN resume_id;
ALTER TABLE skills RENAME COLUMN old_resume_id TO resume_id;
ALTER TABLE languages DROP COLUMN resume_id;
ALTER TABLE languages RENAME COLUMN old_resume_id TO resume_id;
ALTER TABLE education DROP COLUMN resume_id;
ALTER TABLE education RENAME COLUMN old_resume_id TO resume_id;
ALTER TABLE work_experiences DROP COLUMN resume_id;
ALTER TABLE work_experiences RENAME COLUMN old_resume_id TO resume_id;
ALTER TABLE portfolio_projects DROP COLUMN resume_id;
ALTER TABLE portfolio_projects RENAME COLUMN old_resume_id TO resume_id;
ALTER TABLE resumes DROP COLUMN base_resume_id;
ALTER TABLE resumes RENAME COLUMN old_base_resume_id TO base_resume_id;
ALTER TABLE resumes DROP COLUMN id;
ALTER TABLE resumes RENAME COLUMN old_id TO id;

-- 7. restore constraints/indexes on the int columns
ALTER TABLE resumes ADD CONSTRAINT resumes_pkey PRIMARY KEY (id);
ALTER SEQUENCE resumes_old_id_seq RENAME TO resumes_id_seq;
ALTER TABLE skills
    ALTER COLUMN resume_id SET NOT NULL,
    ADD CONSTRAINT skills_resume_id_fkey
        FOREIGN KEY (resume_id) REFERENCES resumes(id) ON DELETE CASCADE;
ALTER TABLE languages
    ALTER COLUMN resume_id SET NOT NULL,
    ADD CONSTRAINT languages_resume_id_fkey
        FOREIGN KEY (resume_id) REFERENCES resumes(id) ON DELETE CASCADE;
ALTER TABLE education
    ALTER COLUMN resume_id SET NOT NULL,
    ADD CONSTRAINT education_resume_id_fkey
        FOREIGN KEY (resume_id) REFERENCES resumes(id) ON DELETE CASCADE;
ALTER TABLE work_experiences
    ALTER COLUMN resume_id SET NOT NULL,
    ADD CONSTRAINT work_experiences_resume_id_fkey
        FOREIGN KEY (resume_id) REFERENCES resumes(id) ON DELETE CASCADE;
ALTER TABLE portfolio_projects
    ALTER COLUMN resume_id SET NOT NULL,
    ADD CONSTRAINT portfolio_projects_resume_id_fkey
        FOREIGN KEY (resume_id) REFERENCES resumes(id) ON DELETE CASCADE;
ALTER TABLE resumes ADD CONSTRAINT resumes_base_resume_id_fkey
    FOREIGN KEY (base_resume_id) REFERENCES resumes(id) ON DELETE NO ACTION;
ALTER TABLE resumes ADD CONSTRAINT resumes_base_resume_id_not_self
    CHECK (base_resume_id IS NULL OR base_resume_id <> id);
CREATE UNIQUE INDEX resumes_email_base_unique
    ON resumes (email) WHERE base_resume_id IS NULL;
CREATE INDEX idx_skills_resume_id ON skills(resume_id);
CREATE INDEX idx_languages_resume_id ON languages(resume_id);
CREATE INDEX idx_education_resume_id ON education(resume_id);
CREATE INDEX idx_work_experiences_resume_id ON work_experiences(resume_id);
CREATE INDEX idx_portfolio_projects_resume_id ON portfolio_projects(resume_id);
