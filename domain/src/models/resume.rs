use crate::schema::resumes;
use chrono::{NaiveDate, NaiveDateTime};
use diesel::prelude::*;
use rocket::serde::de::{Deserializer, Error as DeError};
use rocket::serde::json::Value as JsonValue;
use rocket::serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::ToSchema;

use super::{
    Education, EducationKeyPoint, Framework, Language, PartialDate, PortfolioKeyPoint,
    PortfolioProject, PortfolioTechnology, Skill, WorkExperience, WorkExperienceKeyPoint,
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

fn deserialize_optional_nullable_target_date<'de, D>(
    deserializer: D,
) -> Result<Option<Option<PartialDate>>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = JsonValue::deserialize(deserializer)?;
    match value {
        JsonValue::Null => Ok(Some(None)),
        JsonValue::String(value) if value.trim().is_empty() => Ok(None),
        JsonValue::String(value) => PartialDate::from_iso_str(&value)
            .map(|date| Some(Some(date)))
            .map_err(DeError::custom),
        other => Err(DeError::custom(format!(
            "expected date string or null, got {}",
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
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub created_by: Option<i32>,
    pub is_public: bool,
    pub executive_summary: Option<String>,
    #[schema(max_length = 500)]
    pub video: Option<String>,
    pub base_resume_id: Option<i32>,
    pub company_name: Option<String>,
    pub role_title: Option<String>,
    #[schema(value_type = Option<PartialDate>)]
    pub target_date: Option<NaiveDate>,
    #[schema(ignore)]
    pub target_date_precision: Option<String>,
    pub job_description: Option<String>,
    pub variant_label: Option<String>,
    pub show_variant_tag: bool,
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
    pub base_resume_id: Option<i32>,
    pub company_name: Option<String>,
    pub role_title: Option<String>,
    pub target_date: Option<NaiveDate>,
    pub target_date_precision: Option<String>,
    pub job_description: Option<String>,
    pub variant_label: Option<String>,
    pub show_variant_tag: bool,
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

#[derive(Deserialize, ToSchema)]
#[serde(crate = "rocket::serde")]
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
    #[serde(default, deserialize_with = "deserialize_optional_nullable_string")]
    #[schema(value_type = Option<String>, max_length = 255)]
    pub company_name: Option<Option<String>>,
    #[serde(default, deserialize_with = "deserialize_optional_nullable_string")]
    #[schema(value_type = Option<String>, max_length = 255)]
    pub role_title: Option<Option<String>>,
    #[serde(default, deserialize_with = "deserialize_optional_nullable_string")]
    #[schema(value_type = Option<String>, max_length = 255)]
    pub variant_label: Option<Option<String>>,
    #[serde(default, deserialize_with = "deserialize_optional_nullable_string")]
    #[schema(value_type = Option<String>, max_length = 20000)]
    pub job_description: Option<Option<String>>,
    #[serde(
        default,
        deserialize_with = "deserialize_optional_nullable_target_date"
    )]
    #[schema(value_type = Option<PartialDate>)]
    pub target_date: Option<Option<PartialDate>>,
    pub show_variant_tag: Option<bool>,
}

#[derive(AsChangeset, Debug)]
#[diesel(table_name = resumes)]
pub struct UpdateResumeChangeset {
    pub name: Option<String>,
    pub profile_image_url: Option<String>,
    pub location: Option<String>,
    pub email: Option<String>,
    pub github_url: Option<String>,
    pub mobile_number: Option<String>,
    pub executive_summary: Option<Option<String>>,
    pub video: Option<Option<String>>,
    pub is_public: Option<bool>,
    pub company_name: Option<Option<String>>,
    pub role_title: Option<Option<String>>,
    pub variant_label: Option<Option<String>>,
    pub job_description: Option<Option<String>>,
    pub target_date: Option<Option<NaiveDate>>,
    pub target_date_precision: Option<Option<String>>,
    pub show_variant_tag: Option<bool>,
}

#[derive(Deserialize, ToSchema)]
#[serde(crate = "rocket::serde")]
pub struct NewVariantRequest {
    #[schema(max_length = 255)]
    pub company_name: Option<String>,
    #[schema(max_length = 255)]
    pub role_title: Option<String>,
    pub target_date: Option<PartialDate>,
    #[schema(max_length = 20000)]
    pub job_description: Option<String>,
    #[schema(max_length = 255)]
    pub variant_label: Option<String>,
    pub is_public: Option<bool>,
    pub show_variant_tag: Option<bool>,
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
