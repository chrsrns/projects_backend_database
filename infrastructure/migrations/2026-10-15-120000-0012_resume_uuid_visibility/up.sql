-- Migrate resumes + every resume_id/base_resume_id FK from INTEGER to UUID,
-- and replace resumes.is_public with a 3-state visibility column.
-- Atomic: Diesel runs this file in one transaction.

-- 1. visibility column: map is_public true -> 'public', false -> 'private'
ALTER TABLE resumes ADD COLUMN visibility VARCHAR(10);
UPDATE resumes SET visibility = CASE WHEN is_public THEN 'public' ELSE 'private' END;
ALTER TABLE resumes
    ALTER COLUMN visibility SET NOT NULL,
    ALTER COLUMN visibility SET DEFAULT 'private',
    ADD CONSTRAINT resumes_visibility_check
        CHECK (visibility IN ('private', 'unlisted', 'public'));
DROP INDEX idx_resumes_is_public;
ALTER TABLE resumes DROP COLUMN is_public;
CREATE INDEX idx_resumes_visibility ON resumes(visibility);

-- 2. new UUID identity on resumes (no column default: ids are minted app-side)
ALTER TABLE resumes ADD COLUMN new_id UUID;
UPDATE resumes SET new_id = gen_random_uuid();
ALTER TABLE resumes ALTER COLUMN new_id SET NOT NULL;
ALTER TABLE resumes ADD CONSTRAINT resumes_new_id_key UNIQUE (new_id);

-- 3. new self-FK column for variants
ALTER TABLE resumes ADD COLUMN new_base_resume_id UUID;
UPDATE resumes v SET new_base_resume_id = b.new_id
    FROM resumes b WHERE v.base_resume_id = b.id;

-- 4. new UUID FK columns on every resume_id-bearing child table
ALTER TABLE skills ADD COLUMN new_resume_id UUID;
UPDATE skills c SET new_resume_id = r.new_id FROM resumes r WHERE c.resume_id = r.id;
ALTER TABLE languages ADD COLUMN new_resume_id UUID;
UPDATE languages c SET new_resume_id = r.new_id FROM resumes r WHERE c.resume_id = r.id;
ALTER TABLE education ADD COLUMN new_resume_id UUID;
UPDATE education c SET new_resume_id = r.new_id FROM resumes r WHERE c.resume_id = r.id;
ALTER TABLE work_experiences ADD COLUMN new_resume_id UUID;
UPDATE work_experiences c SET new_resume_id = r.new_id FROM resumes r WHERE c.resume_id = r.id;
ALTER TABLE portfolio_projects ADD COLUMN new_resume_id UUID;
UPDATE portfolio_projects c SET new_resume_id = r.new_id FROM resumes r WHERE c.resume_id = r.id;

-- 5. drop everything that depends on the old int columns
DROP INDEX resumes_email_base_unique;
ALTER TABLE resumes DROP CONSTRAINT resumes_base_resume_id_fkey;
ALTER TABLE resumes DROP CONSTRAINT resumes_base_resume_id_not_self;
ALTER TABLE skills DROP CONSTRAINT skills_resume_id_fkey;
ALTER TABLE languages DROP CONSTRAINT languages_resume_id_fkey;
ALTER TABLE education DROP CONSTRAINT education_resume_id_fkey;
ALTER TABLE work_experiences DROP CONSTRAINT work_experiences_resume_id_fkey;
ALTER TABLE portfolio_projects DROP CONSTRAINT portfolio_projects_resume_id_fkey;
ALTER TABLE resumes DROP CONSTRAINT resumes_pkey;

-- 6. swap columns (idx_*_resume_id indexes drop with their old columns)
ALTER TABLE skills DROP COLUMN resume_id;
ALTER TABLE skills RENAME COLUMN new_resume_id TO resume_id;
ALTER TABLE languages DROP COLUMN resume_id;
ALTER TABLE languages RENAME COLUMN new_resume_id TO resume_id;
ALTER TABLE education DROP COLUMN resume_id;
ALTER TABLE education RENAME COLUMN new_resume_id TO resume_id;
ALTER TABLE work_experiences DROP COLUMN resume_id;
ALTER TABLE work_experiences RENAME COLUMN new_resume_id TO resume_id;
ALTER TABLE portfolio_projects DROP COLUMN resume_id;
ALTER TABLE portfolio_projects RENAME COLUMN new_resume_id TO resume_id;
ALTER TABLE resumes DROP COLUMN base_resume_id;
ALTER TABLE resumes RENAME COLUMN new_base_resume_id TO base_resume_id;
ALTER TABLE resumes DROP COLUMN id;
ALTER TABLE resumes RENAME COLUMN new_id TO id;

-- 7. restore constraints/indexes against the UUID columns
ALTER TABLE resumes ADD CONSTRAINT resumes_pkey PRIMARY KEY (id);
ALTER TABLE resumes DROP CONSTRAINT resumes_new_id_key;
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
