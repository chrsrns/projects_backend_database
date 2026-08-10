use super::PartialDate;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedResume {
    pub name: String,
    pub profile_image_url: Option<String>,
    pub location: Option<String>,
    pub email: String,
    pub github_url: Option<String>,
    pub mobile_number: Option<String>,
    pub executive_summary: Option<String>,
    pub video: Option<String>,
    pub is_public: bool,
    pub education: Vec<ParsedEducation>,
    pub education_key_points: Vec<ParsedEducationKeyPoint>,
    pub skills: Vec<ParsedSkill>,
    pub work_experiences: Vec<ParsedWorkExperience>,
    pub work_experience_key_points: Vec<ParsedWorkExperienceKeyPoint>,
    pub portfolio_projects: Vec<ParsedPortfolioProject>,
    pub portfolio_key_points: Vec<ParsedPortfolioKeyPoint>,
    pub portfolio_technologies: Vec<ParsedPortfolioTechnology>,
    pub languages: Vec<ParsedLanguage>,
    pub frameworks: Vec<ParsedFramework>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedEducation {
    pub education_stage: String,
    pub institution_name: String,
    pub degree: Option<String>,
    pub start_date: PartialDate,
    pub end_date: Option<PartialDate>,
    pub description: Option<String>,
    pub display_order: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedEducationKeyPoint {
    pub education_index: usize,
    pub key_point: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedSkill {
    pub skill_name: String,
    pub confidence_percentage: i32,
    pub display_order: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedWorkExperience {
    pub job_title: String,
    pub company_name: Option<String>,
    pub start_date: PartialDate,
    pub end_date: Option<PartialDate>,
    pub description: Option<String>,
    pub display_order: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedWorkExperienceKeyPoint {
    pub work_experience_index: usize,
    pub key_point: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedPortfolioProject {
    pub project_name: String,
    pub image_url: Option<String>,
    pub project_link: Option<String>,
    pub source_code_link: Option<String>,
    pub video_url: Option<String>,
    pub description: Option<String>,
    pub display_order: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedPortfolioKeyPoint {
    pub portfolio_project_index: usize,
    pub key_point: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedPortfolioTechnology {
    pub portfolio_project_index: usize,
    pub technology_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedLanguage {
    pub language_name: String,
    pub display_order: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedFramework {
    pub language_index: usize,
    pub framework_name: String,
}
