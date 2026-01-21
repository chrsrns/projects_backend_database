use chrono::NaiveDateTime;
use domain::models::{
    Education, EducationKeyPoint, Framework, Language, PortfolioKeyPoint, PortfolioProject,
    PortfolioTechnology, Resume, Skill, User, WorkExperience, WorkExperienceKeyPoint,
};
use rocket::serde::Serialize;

#[derive(Serialize)]
pub enum ResponseBody {
    Message(String),
    Resume(Resume),
    Resumes(Vec<Resume>),
    Skill(Skill),
    Skills(Vec<Skill>),
    Language(Language),
    Languages(Vec<Language>),
    Framework(Framework),
    Frameworks(Vec<Framework>),
    Education(Education),
    Educations(Vec<Education>),
    EducationKeyPoint(EducationKeyPoint),
    EducationKeyPoints(Vec<EducationKeyPoint>),
    WorkExperience(WorkExperience),
    WorkExperiences(Vec<WorkExperience>),
    WorkExperienceKeyPoint(WorkExperienceKeyPoint),
    WorkExperienceKeyPoints(Vec<WorkExperienceKeyPoint>),
    PortfolioProject(PortfolioProject),
    PortfolioProjects(Vec<PortfolioProject>),
    PortfolioKeyPoint(PortfolioKeyPoint),
    PortfolioKeyPoints(Vec<PortfolioKeyPoint>),
    PortfolioTechnology(PortfolioTechnology),
    PortfolioTechnologies(Vec<PortfolioTechnology>),
    User(User),
    AuthToken(AuthTokenResponse),
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct AuthTokenResponse {
    pub token: String,
    pub expires_at: NaiveDateTime,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct Response {
    pub body: ResponseBody,
}
