UPDATE education
SET degree = 'N/A'
WHERE degree IS NULL;

ALTER TABLE education
ALTER COLUMN degree SET NOT NULL;
