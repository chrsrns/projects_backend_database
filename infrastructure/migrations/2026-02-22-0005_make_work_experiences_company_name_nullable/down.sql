UPDATE work_experiences
SET company_name = 'Unknown'
WHERE company_name IS NULL;

ALTER TABLE work_experiences
    ALTER COLUMN company_name SET NOT NULL;
