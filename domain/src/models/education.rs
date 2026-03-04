use crate::schema::{education, education_key_points};
use chrono::{NaiveDate, NaiveDateTime};
use diesel::prelude::*;
use rocket::serde::de::{Deserializer, Error as DeError};
use rocket::serde::json::Value as JsonValue;
use rocket::serde::{Deserialize, Serialize};
use utoipa::ToSchema;

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

#[derive(Queryable, Serialize, ToSchema, Ord, Eq, PartialEq, PartialOrd)]
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

#[derive(Insertable, Deserialize, ToSchema)]
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

#[derive(Deserialize, ToSchema)]
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

#[derive(AsChangeset, Deserialize, ToSchema)]
#[serde(crate = "rocket::serde")]
#[diesel(table_name = education)]
pub struct UpdateEducation {
    pub education_stage: Option<String>,
    pub institution_name: Option<String>,
    #[serde(default, deserialize_with = "deserialize_optional_nullable_string")]
    pub degree: Option<Option<String>>,
    pub start_date: Option<NaiveDate>,
    #[serde(default, deserialize_with = "deserialize_optional_nullable_date")]
    pub end_date: Option<Option<NaiveDate>>,
    #[serde(default, deserialize_with = "deserialize_optional_nullable_string")]
    pub description: Option<Option<String>>,
    pub display_order: Option<i32>,
}

#[derive(Queryable, Serialize, ToSchema, Ord, Eq, PartialEq, PartialOrd)]
pub struct EducationKeyPoint {
    pub id: i32,
    pub education_id: i32,
    pub key_point: String,
    pub display_order: Option<i32>,
    pub created_at: NaiveDateTime,
}

#[derive(Insertable, Deserialize, ToSchema)]
#[serde(crate = "rocket::serde")]
#[diesel(table_name = education_key_points)]
pub struct NewEducationKeyPoint {
    pub education_id: i32,
    pub key_point: String,
    pub display_order: Option<i32>,
}

#[derive(Deserialize, ToSchema)]
#[serde(crate = "rocket::serde")]
pub struct NewEducationKeyPointRequest {
    pub key_point: String,
    pub display_order: Option<i32>,
}

#[derive(AsChangeset, Deserialize, ToSchema)]
#[serde(crate = "rocket::serde")]
#[diesel(table_name = education_key_points)]
pub struct UpdateEducationKeyPoint {
    pub key_point: Option<String>,
    pub display_order: Option<i32>,
}
