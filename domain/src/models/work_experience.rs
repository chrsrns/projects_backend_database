use crate::schema::{work_experience_key_points, work_experiences};
use chrono::{NaiveDate, NaiveDateTime};
use diesel::prelude::*;
use rocket::serde::de::{Deserializer, Error as DeError};
use rocket::serde::json::Value as JsonValue;
use rocket::serde::{Deserialize, Serialize};
use serde::Serializer;
use serde::ser::SerializeStruct;
use std::str::FromStr;
use utoipa::ToSchema;

use super::{DatePrecision, PartialDate, deserialize_optional_nullable_partial_date};

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

#[derive(Queryable, ToSchema, Ord, Eq, PartialEq, PartialOrd, Debug)]
pub struct WorkExperience {
    pub id: i32,
    pub resume_id: i32,
    pub job_title: String,
    pub company_name: Option<String>,
    #[schema(value_type = PartialDate)]
    pub start_date: NaiveDate,
    #[schema(value_type = PartialDate)]
    pub end_date: Option<NaiveDate>,
    pub description: Option<String>,
    pub display_order: Option<i32>,
    pub created_at: NaiveDateTime,
    pub active: bool,
    #[schema(ignore)]
    pub start_date_precision: String,
    #[schema(ignore)]
    pub end_date_precision: Option<String>,
}

impl Serialize for WorkExperience {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("WorkExperience", 10)?;

        state.serialize_field("id", &self.id)?;
        state.serialize_field("resume_id", &self.resume_id)?;
        state.serialize_field("job_title", &self.job_title)?;
        state.serialize_field("company_name", &self.company_name)?;

        let start_precision =
            DatePrecision::from_str(&self.start_date_precision).unwrap_or(DatePrecision::Day);
        let start_view = PartialDate {
            canonical: self.start_date,
            precision: start_precision,
        };
        state.serialize_field("start_date", &start_view)?;

        let end_view = self.end_date.map(|d| {
            let p = self
                .end_date_precision
                .as_deref()
                .and_then(|s| DatePrecision::from_str(s).ok())
                .unwrap_or(DatePrecision::Day);
            PartialDate {
                canonical: d,
                precision: p,
            }
        });
        state.serialize_field("end_date", &end_view)?;

        state.serialize_field("description", &self.description)?;
        state.serialize_field("display_order", &self.display_order)?;
        state.serialize_field("created_at", &self.created_at)?;
        state.serialize_field("active", &self.active)?;

        state.end()
    }
}

#[derive(Insertable, Debug)]
#[diesel(table_name = work_experiences)]
pub struct NewWorkExperience {
    pub resume_id: i32,
    pub job_title: String,
    pub company_name: Option<String>,
    pub start_date: NaiveDate,
    pub start_date_precision: String,
    pub end_date: Option<NaiveDate>,
    pub end_date_precision: Option<String>,
    pub description: Option<String>,
    pub display_order: Option<i32>,
}

#[derive(Deserialize, ToSchema)]
#[serde(crate = "rocket::serde")]
pub struct NewWorkExperienceRequest {
    pub job_title: String,
    pub company_name: Option<String>,
    pub start_date: PartialDate,
    pub end_date: Option<PartialDate>,
    pub description: Option<String>,
    pub display_order: Option<i32>,
}

#[derive(AsChangeset, Debug)]
#[diesel(table_name = work_experiences)]
pub struct UpdateWorkExperience {
    pub job_title: Option<String>,
    pub company_name: Option<Option<String>>,
    pub start_date: Option<NaiveDate>,
    pub start_date_precision: Option<String>,
    pub end_date: Option<Option<NaiveDate>>,
    pub end_date_precision: Option<Option<String>>,
    pub description: Option<Option<String>>,
    pub display_order: Option<i32>,
    pub active: Option<bool>,
}

#[derive(Deserialize, ToSchema)]
#[serde(crate = "rocket::serde")]
pub struct UpdateWorkExperienceRequest {
    pub job_title: Option<String>,
    #[serde(default, deserialize_with = "deserialize_optional_nullable_string")]
    pub company_name: Option<Option<String>>,
    pub start_date: Option<PartialDate>,
    #[serde(
        default,
        deserialize_with = "deserialize_optional_nullable_partial_date"
    )]
    pub end_date: Option<Option<PartialDate>>,
    #[serde(default, deserialize_with = "deserialize_optional_nullable_string")]
    pub description: Option<Option<String>>,
    pub display_order: Option<i32>,
    pub active: Option<bool>,
}

#[derive(Queryable, Serialize, ToSchema, Ord, Eq, PartialEq, PartialOrd)]
pub struct WorkExperienceKeyPoint {
    pub id: i32,
    pub work_experience_id: i32,
    pub key_point: String,
    pub display_order: Option<i32>,
    pub created_at: NaiveDateTime,
    pub active: bool,
}

#[derive(Insertable, Deserialize, ToSchema)]
#[serde(crate = "rocket::serde")]
#[diesel(table_name = work_experience_key_points)]
pub struct NewWorkExperienceKeyPoint {
    pub work_experience_id: i32,
    pub key_point: String,
    pub display_order: Option<i32>,
}

#[derive(Deserialize, ToSchema)]
#[serde(crate = "rocket::serde")]
pub struct NewWorkExperienceKeyPointRequest {
    pub key_point: String,
    pub display_order: Option<i32>,
}

#[derive(AsChangeset, Deserialize, ToSchema)]
#[serde(crate = "rocket::serde")]
#[diesel(table_name = work_experience_key_points)]
pub struct UpdateWorkExperienceKeyPoint {
    pub key_point: Option<String>,
    pub display_order: Option<i32>,
    pub active: Option<bool>,
}
