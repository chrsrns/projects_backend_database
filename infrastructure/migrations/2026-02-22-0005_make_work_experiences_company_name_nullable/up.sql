ALTER TABLE work_experiences
    ALTER COLUMN company_name DROP NOT NULL;

UPDATE work_experiences
SET company_name = NULL
WHERE btrim(company_name) = '';
