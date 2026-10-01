ALTER TABLE resumes
    ADD COLUMN base_resume_id INTEGER REFERENCES resumes(id) ON DELETE NO ACTION,
    ADD COLUMN company_name VARCHAR(255),
    ADD COLUMN role_title VARCHAR(255),
    ADD COLUMN target_date DATE,
    ADD COLUMN target_date_precision VARCHAR(10) CHECK (target_date_precision IN ('day', 'month', 'year')),
    ADD COLUMN job_description TEXT,
    ADD COLUMN variant_label VARCHAR(255),
    ADD COLUMN show_variant_tag BOOLEAN NOT NULL DEFAULT TRUE;

ALTER TABLE resumes
    ADD CONSTRAINT resumes_target_date_precision_requires_date
    CHECK (target_date IS NOT NULL OR target_date_precision IS NULL);

ALTER TABLE resumes
    ADD CONSTRAINT resumes_base_resume_id_not_self
    CHECK (base_resume_id IS NULL OR base_resume_id <> id);

ALTER TABLE resumes DROP CONSTRAINT IF EXISTS resumes_email_key;

CREATE UNIQUE INDEX resumes_email_base_unique
    ON resumes (email)
    WHERE base_resume_id IS NULL;
