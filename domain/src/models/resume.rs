use crate::schema::{
    education, education_key_points, frameworks, languages, portfolio_key_points,
    portfolio_projects, portfolio_technologies, resumes, skills, work_experience_key_points,
    work_experiences,
};
use chrono::{NaiveDate, NaiveDateTime};
use diesel::prelude::*;
use rocket::serde::de::{Deserializer, Error as DeError};
use rocket::serde::json::Value as JsonValue;
use rocket::serde::{Deserialize, Serialize};
use std::cmp::{Eq, Ord, PartialEq, PartialOrd};

fn deserialize_optional_nullable_string<'de, D>(
    deserializer: D,
) -> Result<Option<Option<String>>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = JsonValue::deserialize(deserializer)?;
    match value {
        JsonValue::Null => Ok(Some(None)),
        JsonValue::String(v) => Ok(Some(Some(v))),
        other => Err(DeError::custom(format!(
            "expected string or null, got {}",
            other
        ))),
    }
}

fn deserialize_optional_nullable_date<'de, D>(
    deserializer: D,
) -> Result<Option<Option<NaiveDate>>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = JsonValue::deserialize(deserializer)?;
    match value {
        JsonValue::Null => Ok(Some(None)),
        JsonValue::String(v) => NaiveDate::parse_from_str(&v, "%Y-%m-%d")
            .map(Some)
            .map(Some)
            .map_err(|e| DeError::custom(format!("invalid date '{}': {}", v, e))),
        other => Err(DeError::custom(format!(
            "expected date string or null, got {}",
            other
        ))),
    }
}

#[derive(Queryable, Serialize, Ord, Eq, PartialEq, PartialOrd)]
pub struct Resume {
    pub id: i32,
    pub name: String,
    pub profile_image_url: Option<String>,
    pub location: Option<String>,
    pub email: String,
    pub github_url: Option<String>,
    pub mobile_number: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub created_by: Option<i32>,
    pub is_public: bool,
}

#[derive(Insertable, Deserialize)]
#[serde(crate = "rocket::serde")]
#[diesel(table_name = resumes)]
pub struct NewResume {
    pub name: String,
    pub profile_image_url: Option<String>,
    pub location: Option<String>,
    pub email: String,
    pub github_url: Option<String>,
    pub mobile_number: Option<String>,
    pub created_by: Option<i32>,
    pub is_public: bool,
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct NewResumeRequest {
    pub name: String,
    pub profile_image_url: Option<String>,
    pub location: Option<String>,
    pub email: String,
    pub github_url: Option<String>,
    pub mobile_number: Option<String>,
    pub is_public: Option<bool>,
}

#[derive(AsChangeset, Deserialize)]
#[serde(crate = "rocket::serde")]
#[diesel(table_name = resumes)]
pub struct UpdateResume {
    pub name: Option<String>,
    pub profile_image_url: Option<String>,
    pub location: Option<String>,
    pub email: Option<String>,
    pub github_url: Option<String>,
    pub mobile_number: Option<String>,
    pub is_public: Option<bool>,
}

// ==================== Skills ====================

#[derive(Queryable, Serialize, Ord, Eq, PartialEq, PartialOrd)]
pub struct Skill {
    pub id: i32,
    pub resume_id: i32,
    pub skill_name: String,
    pub confidence_percentage: i32,
    pub display_order: Option<i32>,
    pub created_at: NaiveDateTime,
}

#[derive(Insertable, Deserialize)]
#[serde(crate = "rocket::serde")]
#[diesel(table_name = skills)]
pub struct NewSkill {
    pub resume_id: i32,
    pub skill_name: String,
    pub confidence_percentage: i32,
    pub display_order: Option<i32>,
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct NewSkillRequest {
    pub skill_name: String,
    pub confidence_percentage: i32,
    pub display_order: Option<i32>,
}

#[derive(AsChangeset, Deserialize)]
#[serde(crate = "rocket::serde")]
#[diesel(table_name = skills)]
pub struct UpdateSkill {
    pub skill_name: Option<String>,
    pub confidence_percentage: Option<i32>,
    pub display_order: Option<i32>,
}

// ==================== Languages ====================

#[derive(Queryable, Serialize, Ord, Eq, PartialEq, PartialOrd)]
pub struct Language {
    pub id: i32,
    pub resume_id: i32,
    pub language_name: String,
    pub display_order: Option<i32>,
    pub created_at: NaiveDateTime,
}

#[derive(Insertable, Deserialize)]
#[serde(crate = "rocket::serde")]
#[diesel(table_name = languages)]
pub struct NewLanguage {
    pub resume_id: i32,
    pub language_name: String,
    pub display_order: Option<i32>,
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct NewLanguageRequest {
    pub language_name: String,
    pub display_order: Option<i32>,
}

#[derive(AsChangeset, Deserialize)]
#[serde(crate = "rocket::serde")]
#[diesel(table_name = languages)]
pub struct UpdateLanguage {
    pub language_name: Option<String>,
    pub display_order: Option<i32>,
}

// ==================== Frameworks ====================

#[derive(Queryable, Serialize, Ord, Eq, PartialEq, PartialOrd)]
pub struct Framework {
    pub id: i32,
    pub language_id: i32,
    pub framework_name: String,
    pub display_order: Option<i32>,
    pub created_at: NaiveDateTime,
}

#[derive(Insertable, Deserialize)]
#[serde(crate = "rocket::serde")]
#[diesel(table_name = frameworks)]
pub struct NewFramework {
    pub language_id: i32,
    pub framework_name: String,
    pub display_order: Option<i32>,
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct NewFrameworkRequest {
    pub framework_name: String,
    pub display_order: Option<i32>,
}

#[derive(AsChangeset, Deserialize)]
#[serde(crate = "rocket::serde")]
#[diesel(table_name = frameworks)]
pub struct UpdateFramework {
    pub framework_name: Option<String>,
    pub display_order: Option<i32>,
}

// ==================== Education ====================

#[derive(Queryable, Serialize, Ord, Eq, PartialEq, PartialOrd)]
pub struct Education {
    pub id: i32,
    pub resume_id: i32,
    pub education_stage: String,
    pub institution_name: String,
    pub degree: Option<String>,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub description: Option<String>,
    pub display_order: Option<i32>,
    pub created_at: NaiveDateTime,
}

#[derive(Insertable, Deserialize)]
#[serde(crate = "rocket::serde")]
#[diesel(table_name = education)]
pub struct NewEducation {
    pub resume_id: i32,
    pub education_stage: String,
    pub institution_name: String,
    pub degree: Option<String>,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub description: Option<String>,
    pub display_order: Option<i32>,
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct NewEducationRequest {
    pub education_stage: String,
    pub institution_name: String,
    pub degree: Option<String>,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub description: Option<String>,
    pub display_order: Option<i32>,
}

#[derive(AsChangeset, Deserialize)]
#[serde(crate = "rocket::serde")]
#[diesel(table_name = education)]
pub struct UpdateEducation {
    pub education_stage: Option<String>,
    pub institution_name: Option<String>,
    pub degree: Option<Option<String>>,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<Option<NaiveDate>>,
    pub description: Option<Option<String>>,
    pub display_order: Option<i32>,
}

// ==================== Education Key Points ====================

#[derive(Queryable, Serialize, Ord, Eq, PartialEq, PartialOrd)]
pub struct EducationKeyPoint {
    pub id: i32,
    pub education_id: i32,
    pub key_point: String,
    pub display_order: Option<i32>,
    pub created_at: NaiveDateTime,
}

#[derive(Insertable, Deserialize)]
#[serde(crate = "rocket::serde")]
#[diesel(table_name = education_key_points)]
pub struct NewEducationKeyPoint {
    pub education_id: i32,
    pub key_point: String,
    pub display_order: Option<i32>,
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct NewEducationKeyPointRequest {
    pub key_point: String,
    pub display_order: Option<i32>,
}

#[derive(AsChangeset, Deserialize)]
#[serde(crate = "rocket::serde")]
#[diesel(table_name = education_key_points)]
pub struct UpdateEducationKeyPoint {
    pub key_point: Option<String>,
    pub display_order: Option<i32>,
}

// ==================== Work Experiences ====================

#[derive(Queryable, Serialize, Ord, Eq, PartialEq, PartialOrd)]
pub struct WorkExperience {
    pub id: i32,
    pub resume_id: i32,
    pub job_title: String,
    pub company_name: Option<String>,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub description: Option<String>,
    pub display_order: Option<i32>,
    pub created_at: NaiveDateTime,
}

#[derive(Insertable, Deserialize)]
#[serde(crate = "rocket::serde")]
#[diesel(table_name = work_experiences)]
pub struct NewWorkExperience {
    pub resume_id: i32,
    pub job_title: String,
    pub company_name: Option<String>,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub description: Option<String>,
    pub display_order: Option<i32>,
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct NewWorkExperienceRequest {
    pub job_title: String,
    pub company_name: Option<String>,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub description: Option<String>,
    pub display_order: Option<i32>,
}

#[derive(AsChangeset, Deserialize)]
#[serde(crate = "rocket::serde")]
#[diesel(table_name = work_experiences)]
pub struct UpdateWorkExperience {
    pub job_title: Option<String>,
    #[serde(default, deserialize_with = "deserialize_optional_nullable_string")]
    pub company_name: Option<Option<String>>,
    pub start_date: Option<NaiveDate>,
    #[serde(default, deserialize_with = "deserialize_optional_nullable_date")]
    pub end_date: Option<Option<NaiveDate>>,
    #[serde(default, deserialize_with = "deserialize_optional_nullable_string")]
    pub description: Option<Option<String>>,
    pub display_order: Option<i32>,
}

// ==================== Work Experience Key Points ====================

#[derive(Queryable, Serialize, Ord, Eq, PartialEq, PartialOrd)]
pub struct WorkExperienceKeyPoint {
    pub id: i32,
    pub work_experience_id: i32,
    pub key_point: String,
    pub display_order: Option<i32>,
    pub created_at: NaiveDateTime,
}

#[derive(Insertable, Deserialize)]
#[serde(crate = "rocket::serde")]
#[diesel(table_name = work_experience_key_points)]
pub struct NewWorkExperienceKeyPoint {
    pub work_experience_id: i32,
    pub key_point: String,
    pub display_order: Option<i32>,
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct NewWorkExperienceKeyPointRequest {
    pub key_point: String,
    pub display_order: Option<i32>,
}

#[derive(AsChangeset, Deserialize)]
#[serde(crate = "rocket::serde")]
#[diesel(table_name = work_experience_key_points)]
pub struct UpdateWorkExperienceKeyPoint {
    pub key_point: Option<String>,
    pub display_order: Option<i32>,
}

// ==================== Portfolio Projects ====================

#[derive(Queryable, Serialize, Ord, Eq, PartialEq, PartialOrd)]
pub struct PortfolioProject {
    pub id: i32,
    pub resume_id: i32,
    pub project_name: String,
    pub image_url: Option<String>,
    pub project_link: Option<String>,
    pub source_code_link: Option<String>,
    pub description: Option<String>,
    pub display_order: Option<i32>,
    pub created_at: NaiveDateTime,
}

#[derive(Insertable, Deserialize)]
#[serde(crate = "rocket::serde")]
#[diesel(table_name = portfolio_projects)]
pub struct NewPortfolioProject {
    pub resume_id: i32,
    pub project_name: String,
    pub image_url: Option<String>,
    pub project_link: Option<String>,
    pub source_code_link: Option<String>,
    pub description: Option<String>,
    pub display_order: Option<i32>,
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct NewPortfolioProjectRequest {
    pub project_name: String,
    pub image_url: Option<String>,
    pub project_link: Option<String>,
    pub source_code_link: Option<String>,
    pub description: Option<String>,
    pub display_order: Option<i32>,
}

#[derive(AsChangeset, Deserialize)]
#[serde(crate = "rocket::serde")]
#[diesel(table_name = portfolio_projects)]
pub struct UpdatePortfolioProject {
    pub project_name: Option<String>,
    pub image_url: Option<Option<String>>,
    pub project_link: Option<Option<String>>,
    pub source_code_link: Option<Option<String>>,
    pub description: Option<Option<String>>,
    pub display_order: Option<i32>,
}

// ==================== Portfolio Key Points ====================

#[derive(Queryable, Serialize, Ord, Eq, PartialEq, PartialOrd)]
pub struct PortfolioKeyPoint {
    pub id: i32,
    pub portfolio_project_id: i32,
    pub key_point: String,
    pub display_order: Option<i32>,
    pub created_at: NaiveDateTime,
}

#[derive(Insertable, Deserialize)]
#[serde(crate = "rocket::serde")]
#[diesel(table_name = portfolio_key_points)]
pub struct NewPortfolioKeyPoint {
    pub portfolio_project_id: i32,
    pub key_point: String,
    pub display_order: Option<i32>,
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct NewPortfolioKeyPointRequest {
    pub key_point: String,
    pub display_order: Option<i32>,
}

#[derive(AsChangeset, Deserialize)]
#[serde(crate = "rocket::serde")]
#[diesel(table_name = portfolio_key_points)]
pub struct UpdatePortfolioKeyPoint {
    pub key_point: Option<String>,
    pub display_order: Option<i32>,
}

// ==================== Portfolio Technologies ====================

#[derive(Queryable, Serialize, Ord, Eq, PartialEq, PartialOrd)]
pub struct PortfolioTechnology {
    pub id: i32,
    pub portfolio_project_id: i32,
    pub technology_name: String,
    pub display_order: Option<i32>,
    pub created_at: NaiveDateTime,
}

#[derive(Insertable, Deserialize)]
#[serde(crate = "rocket::serde")]
#[diesel(table_name = portfolio_technologies)]
pub struct NewPortfolioTechnology {
    pub portfolio_project_id: i32,
    pub technology_name: String,
    pub display_order: Option<i32>,
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct NewPortfolioTechnologyRequest {
    pub technology_name: String,
    pub display_order: Option<i32>,
}

#[derive(AsChangeset, Deserialize)]
#[serde(crate = "rocket::serde")]
#[diesel(table_name = portfolio_technologies)]
pub struct UpdatePortfolioTechnology {
    pub technology_name: Option<String>,
    pub display_order: Option<i32>,
}
