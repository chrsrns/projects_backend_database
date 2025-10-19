-- Create resumes table (main entity)
CREATE TABLE resumes (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    profile_image_url VARCHAR(500),
    location VARCHAR(255),
    email VARCHAR(255) NOT NULL UNIQUE,
    github_url VARCHAR(500),
    mobile_number VARCHAR(50),
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Create skills table
CREATE TABLE skills (
    id SERIAL PRIMARY KEY,
    resume_id INTEGER NOT NULL REFERENCES resumes(id) ON DELETE CASCADE,
    skill_name VARCHAR(255) NOT NULL,
    confidence_percentage INTEGER NOT NULL CHECK (confidence_percentage >= 0 AND confidence_percentage <= 100),
    display_order INTEGER DEFAULT 0,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Create languages table (programming languages)
CREATE TABLE languages (
    id SERIAL PRIMARY KEY,
    resume_id INTEGER NOT NULL REFERENCES resumes(id) ON DELETE CASCADE,
    language_name VARCHAR(255) NOT NULL,
    display_order INTEGER DEFAULT 0,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Create frameworks table (associated with languages)
CREATE TABLE frameworks (
    id SERIAL PRIMARY KEY,
    language_id INTEGER NOT NULL REFERENCES languages(id) ON DELETE CASCADE,
    framework_name VARCHAR(255) NOT NULL,
    display_order INTEGER DEFAULT 0,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Create education table
CREATE TABLE education (
    id SERIAL PRIMARY KEY,
    resume_id INTEGER NOT NULL REFERENCES resumes(id) ON DELETE CASCADE,
    education_stage VARCHAR(100) NOT NULL,
    institution_name VARCHAR(255) NOT NULL,
    degree VARCHAR(255) NOT NULL,
    start_date DATE NOT NULL,
    end_date DATE,
    description TEXT,
    display_order INTEGER DEFAULT 0,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Create education_key_points table
CREATE TABLE education_key_points (
    id SERIAL PRIMARY KEY,
    education_id INTEGER NOT NULL REFERENCES education(id) ON DELETE CASCADE,
    key_point TEXT NOT NULL,
    display_order INTEGER DEFAULT 0,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Create work_experiences table
CREATE TABLE work_experiences (
    id SERIAL PRIMARY KEY,
    resume_id INTEGER NOT NULL REFERENCES resumes(id) ON DELETE CASCADE,
    job_title VARCHAR(255) NOT NULL,
    company_name VARCHAR(255) NOT NULL,
    start_date DATE NOT NULL,
    end_date DATE,
    description TEXT,
    display_order INTEGER DEFAULT 0,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Create work_experience_key_points table
CREATE TABLE work_experience_key_points (
    id SERIAL PRIMARY KEY,
    work_experience_id INTEGER NOT NULL REFERENCES work_experiences(id) ON DELETE CASCADE,
    key_point TEXT NOT NULL,
    display_order INTEGER DEFAULT 0,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Create portfolio_projects table
CREATE TABLE portfolio_projects (
    id SERIAL PRIMARY KEY,
    resume_id INTEGER NOT NULL REFERENCES resumes(id) ON DELETE CASCADE,
    project_name VARCHAR(255) NOT NULL,
    image_url VARCHAR(500),
    project_link VARCHAR(500),
    source_code_link VARCHAR(500),
    description TEXT,
    display_order INTEGER DEFAULT 0,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Create portfolio_key_points table
CREATE TABLE portfolio_key_points (
    id SERIAL PRIMARY KEY,
    portfolio_project_id INTEGER NOT NULL REFERENCES portfolio_projects(id) ON DELETE CASCADE,
    key_point TEXT NOT NULL,
    display_order INTEGER DEFAULT 0,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Create portfolio_technologies table
CREATE TABLE portfolio_technologies (
    id SERIAL PRIMARY KEY,
    portfolio_project_id INTEGER NOT NULL REFERENCES portfolio_projects(id) ON DELETE CASCADE,
    technology_name VARCHAR(255) NOT NULL,
    display_order INTEGER DEFAULT 0,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Create indexes for efficient queries
CREATE INDEX idx_skills_resume_id ON skills(resume_id);
CREATE INDEX idx_languages_resume_id ON languages(resume_id);
CREATE INDEX idx_frameworks_language_id ON frameworks(language_id);
CREATE INDEX idx_education_resume_id ON education(resume_id);
CREATE INDEX idx_education_key_points_education_id ON education_key_points(education_id);
CREATE INDEX idx_work_experiences_resume_id ON work_experiences(resume_id);
CREATE INDEX idx_work_experience_key_points_work_experience_id ON work_experience_key_points(work_experience_id);
CREATE INDEX idx_portfolio_projects_resume_id ON portfolio_projects(resume_id);
CREATE INDEX idx_portfolio_key_points_portfolio_project_id ON portfolio_key_points(portfolio_project_id);
CREATE INDEX idx_portfolio_technologies_portfolio_project_id ON portfolio_technologies(portfolio_project_id);

-- Set up automatic updated_at timestamp management for resumes table
SELECT diesel_manage_updated_at('resumes');
