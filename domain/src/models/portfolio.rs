use crate::schema::{portfolio_key_points, portfolio_projects, portfolio_technologies};
use chrono::NaiveDateTime;
use diesel::prelude::*;
use rocket::serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use super::deserialize_optional_nullable_string;

#[derive(Queryable, Serialize, ToSchema, Ord, Eq, PartialEq, PartialOrd)]
pub struct PortfolioProject {
    pub id: i32,
    pub project_name: String,
    #[schema(max_length = 500)]
    pub image_url: Option<String>,
    #[schema(max_length = 500)]
    pub project_link: Option<String>,
    #[schema(max_length = 500)]
    pub source_code_link: Option<String>,
    #[schema(max_length = 500)]
    pub description: Option<String>,
    pub display_order: Option<i32>,
    pub created_at: NaiveDateTime,
    pub active: bool,
    #[schema(max_length = 500)]
    pub video_url: Option<String>,
    pub resume_id: Uuid,
}

#[derive(Insertable, Deserialize, ToSchema)]
#[serde(crate = "rocket::serde")]
#[diesel(table_name = portfolio_projects)]
pub struct NewPortfolioProject {
    pub resume_id: Uuid,
    pub project_name: String,
    #[schema(max_length = 500)]
    pub image_url: Option<String>,
    #[schema(max_length = 500)]
    pub project_link: Option<String>,
    #[schema(max_length = 500)]
    pub source_code_link: Option<String>,
    #[schema(max_length = 500)]
    pub video_url: Option<String>,
    #[schema(max_length = 500)]
    pub description: Option<String>,
    pub display_order: Option<i32>,
    pub active: bool,
}

#[derive(Deserialize, ToSchema)]
#[serde(crate = "rocket::serde")]
pub struct NewPortfolioProjectRequest {
    pub project_name: String,
    #[schema(max_length = 500)]
    pub image_url: Option<String>,
    #[schema(max_length = 500)]
    pub project_link: Option<String>,
    #[schema(max_length = 500)]
    pub source_code_link: Option<String>,
    #[schema(max_length = 500)]
    pub video_url: Option<String>,
    #[schema(max_length = 500)]
    pub description: Option<String>,
    pub display_order: Option<i32>,
}

#[derive(AsChangeset, Deserialize, ToSchema)]
#[serde(crate = "rocket::serde")]
#[diesel(table_name = portfolio_projects)]
pub struct UpdatePortfolioProject {
    pub project_name: Option<String>,
    #[serde(default, deserialize_with = "deserialize_optional_nullable_string")]
    #[schema(value_type = Option<String>, max_length = 500)]
    pub image_url: Option<Option<String>>,
    #[serde(default, deserialize_with = "deserialize_optional_nullable_string")]
    #[schema(value_type = Option<String>, max_length = 500)]
    pub project_link: Option<Option<String>>,
    #[serde(default, deserialize_with = "deserialize_optional_nullable_string")]
    #[schema(value_type = Option<String>, max_length = 500)]
    pub source_code_link: Option<Option<String>>,
    #[serde(default, deserialize_with = "deserialize_optional_nullable_string")]
    #[schema(value_type = Option<String>, max_length = 500)]
    pub video_url: Option<Option<String>>,
    #[serde(default, deserialize_with = "deserialize_optional_nullable_string")]
    #[schema(value_type = Option<String>, max_length = 500)]
    pub description: Option<Option<String>>,
    pub display_order: Option<i32>,
    pub active: Option<bool>,
}

impl UpdatePortfolioProject {
    /// True when every patch field is absent — the payload carries no
    /// change. Must be called after normalization so fields normalized
    /// to absent (e.g. an empty string) count as no change.
    pub fn is_noop(&self) -> bool {
        self.project_name.is_none()
            && self.image_url.is_none()
            && self.project_link.is_none()
            && self.source_code_link.is_none()
            && self.video_url.is_none()
            && self.description.is_none()
            && self.display_order.is_none()
            && self.active.is_none()
    }
}

#[derive(Queryable, Serialize, ToSchema, Ord, Eq, PartialEq, PartialOrd)]
pub struct PortfolioKeyPoint {
    pub id: i32,
    pub portfolio_project_id: i32,
    pub key_point: String,
    pub display_order: Option<i32>,
    pub created_at: NaiveDateTime,
    pub active: bool,
}

#[derive(Insertable, Deserialize, ToSchema)]
#[serde(crate = "rocket::serde")]
#[diesel(table_name = portfolio_key_points)]
pub struct NewPortfolioKeyPoint {
    pub portfolio_project_id: i32,
    pub key_point: String,
    pub display_order: Option<i32>,
    pub active: bool,
}

#[derive(Deserialize, ToSchema)]
#[serde(crate = "rocket::serde")]
pub struct NewPortfolioKeyPointRequest {
    pub key_point: String,
    pub display_order: Option<i32>,
}

#[derive(AsChangeset, Deserialize, ToSchema)]
#[serde(crate = "rocket::serde")]
#[diesel(table_name = portfolio_key_points)]
pub struct UpdatePortfolioKeyPoint {
    pub key_point: Option<String>,
    pub display_order: Option<i32>,
    pub active: Option<bool>,
}

#[derive(Queryable, Serialize, ToSchema, Ord, Eq, PartialEq, PartialOrd)]
pub struct PortfolioTechnology {
    pub id: i32,
    pub portfolio_project_id: i32,
    pub technology_name: String,
    pub display_order: Option<i32>,
    pub created_at: NaiveDateTime,
    pub active: bool,
}

#[derive(Insertable, Deserialize, ToSchema)]
#[serde(crate = "rocket::serde")]
#[diesel(table_name = portfolio_technologies)]
pub struct NewPortfolioTechnology {
    pub portfolio_project_id: i32,
    pub technology_name: String,
    pub display_order: Option<i32>,
    pub active: bool,
}

#[derive(Deserialize, ToSchema)]
#[serde(crate = "rocket::serde")]
pub struct NewPortfolioTechnologyRequest {
    pub technology_name: String,
    pub display_order: Option<i32>,
}

#[derive(AsChangeset, Deserialize, ToSchema)]
#[serde(crate = "rocket::serde")]
#[diesel(table_name = portfolio_technologies)]
pub struct UpdatePortfolioTechnology {
    pub technology_name: Option<String>,
    pub display_order: Option<i32>,
    pub active: Option<bool>,
}
