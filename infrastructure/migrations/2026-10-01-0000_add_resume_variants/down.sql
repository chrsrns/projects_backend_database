DROP INDEX IF EXISTS resumes_email_base_unique;

ALTER TABLE resumes ADD CONSTRAINT resumes_email_key UNIQUE (email);

ALTER TABLE resumes DROP CONSTRAINT IF EXISTS resumes_base_resume_id_not_self;

ALTER TABLE resumes DROP CONSTRAINT IF EXISTS resumes_target_date_precision_requires_date;

ALTER TABLE resumes
    DROP COLUMN show_variant_tag,
    DROP COLUMN variant_label,
    DROP COLUMN job_description,
    DROP COLUMN target_date_precision,
    DROP COLUMN target_date,
    DROP COLUMN role_title,
    DROP COLUMN company_name,
    DROP COLUMN base_resume_id;
