ALTER TABLE education
    ADD COLUMN start_date_precision VARCHAR(10) NOT NULL DEFAULT 'day' CHECK (start_date_precision IN ('day', 'month', 'year')),
    ADD COLUMN end_date_precision VARCHAR(10) CHECK (end_date_precision IN ('day', 'month', 'year'));

UPDATE education
SET end_date_precision = 'day'
WHERE end_date IS NOT NULL;

ALTER TABLE work_experiences
    ADD COLUMN start_date_precision VARCHAR(10) NOT NULL DEFAULT 'day' CHECK (start_date_precision IN ('day', 'month', 'year')),
    ADD COLUMN end_date_precision VARCHAR(10) CHECK (end_date_precision IN ('day', 'month', 'year'));

UPDATE work_experiences
SET end_date_precision = 'day'
WHERE end_date IS NOT NULL;
