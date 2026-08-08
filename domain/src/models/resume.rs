use crate::schema::resumes;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use rocket::serde::de::{Deserializer, Error as DeError};
use rocket::serde::json::Value as JsonValue;
use rocket::serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::ToSchema;

use super::{
    Education, EducationKeyPoint, Framework, Language, PortfolioKeyPoint, PortfolioProject,
    PortfolioTechnology, Skill, WorkExperience, WorkExperienceKeyPoint,
};

fn deserialize_optional_nullable_string<'de, D>(
    deserializer: D,
) -> Result<Option<Option<String>>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = JsonValue::deserialize(deserializer)?;
    match value {
        JsonValue::Null => Ok(Some(None)),
        JsonValue::String(value) => Ok(Some(Some(value))),
        other => Err(DeError::custom(format!(
            "expected string or null, got {}",
            other
        ))),
    }
}

#[derive(Queryable, Serialize, ToSchema, Ord, Eq, PartialEq, PartialOrd)]
pub struct Resume {
    pub id: i32,
    pub name: String,
    pub profile_image_url: Option<String>,
    pub location: Option<String>,
    pub email: String,
    pub github_url: Option<String>,
    pub mobile_number: Option<String>,
    pub executive_summary: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub created_by: Option<i32>,
    pub is_public: bool,
    #[schema(max_length = 500)]
    pub video: Option<String>,
}

#[derive(Insertable, Deserialize, ToSchema)]
#[serde(crate = "rocket::serde")]
#[diesel(table_name = resumes)]
pub struct NewResume {
    pub name: String,
    pub profile_image_url: Option<String>,
    pub location: Option<String>,
    pub email: String,
    pub github_url: Option<String>,
    pub mobile_number: Option<String>,
    pub executive_summary: Option<String>,
    pub created_by: Option<i32>,
    pub is_public: bool,
    #[schema(max_length = 500)]
    pub video: Option<String>,
}

#[derive(Deserialize, ToSchema)]
#[serde(crate = "rocket::serde")]
pub struct NewResumeRequest {
    pub name: String,
    pub profile_image_url: Option<String>,
    pub location: Option<String>,
    pub email: String,
    pub github_url: Option<String>,
    pub mobile_number: Option<String>,
    pub executive_summary: Option<String>,
    pub is_public: Option<bool>,
    #[schema(max_length = 500)]
    pub video: Option<String>,
}

#[derive(AsChangeset, Deserialize, ToSchema)]
#[serde(crate = "rocket::serde")]
#[diesel(table_name = resumes)]
pub struct UpdateResume {
    pub name: Option<String>,
    pub profile_image_url: Option<String>,
    pub location: Option<String>,
    pub email: Option<String>,
    pub github_url: Option<String>,
    pub mobile_number: Option<String>,
    #[serde(default, deserialize_with = "deserialize_optional_nullable_string")]
    pub executive_summary: Option<Option<String>>,
    #[serde(default, deserialize_with = "deserialize_optional_nullable_string")]
    #[schema(value_type = Option<String>, max_length = 500)]
    pub video: Option<Option<String>>,
    pub is_public: Option<bool>,
}

#[derive(Serialize, ToSchema)]
#[serde(crate = "rocket::serde")]
pub struct FullResume {
    pub resume: Resume,
    pub education: Vec<Education>,
    pub education_key_points: HashMap<i32, Vec<EducationKeyPoint>>,
    pub skills: Vec<Skill>,
    pub work_experiences: Vec<WorkExperience>,
    pub work_experience_key_points: HashMap<i32, Vec<WorkExperienceKeyPoint>>,
    pub portfolio_projects: Vec<PortfolioProject>,
    pub portfolio_key_points: HashMap<i32, Vec<PortfolioKeyPoint>>,
    pub portfolio_technologies: HashMap<i32, Vec<PortfolioTechnology>>,
    pub languages: Vec<Language>,
    pub frameworks: HashMap<i32, Vec<Framework>>,
}
