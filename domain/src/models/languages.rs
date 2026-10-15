use crate::schema::languages;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use rocket::serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Queryable, Serialize, ToSchema, Ord, Eq, PartialEq, PartialOrd)]
pub struct Language {
    pub id: i32,
    pub language_name: String,
    pub display_order: Option<i32>,
    pub created_at: NaiveDateTime,
    pub resume_id: Uuid,
}

#[derive(Insertable, Deserialize, ToSchema)]
#[serde(crate = "rocket::serde")]
#[diesel(table_name = languages)]
pub struct NewLanguage {
    pub resume_id: Uuid,
    pub language_name: String,
    pub display_order: Option<i32>,
}

#[derive(Deserialize, ToSchema)]
#[serde(crate = "rocket::serde")]
pub struct NewLanguageRequest {
    pub language_name: String,
    pub display_order: Option<i32>,
}

#[derive(AsChangeset, Deserialize, ToSchema)]
#[serde(crate = "rocket::serde")]
#[diesel(table_name = languages)]
pub struct UpdateLanguage {
    pub language_name: Option<String>,
    pub display_order: Option<i32>,
}
