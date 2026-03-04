use crate::schema::frameworks;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use rocket::serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Queryable, Serialize, ToSchema, Ord, Eq, PartialEq, PartialOrd)]
pub struct Framework {
    pub id: i32,
    pub language_id: i32,
    pub framework_name: String,
    pub display_order: Option<i32>,
    pub created_at: NaiveDateTime,
}

#[derive(Insertable, Deserialize, ToSchema)]
#[serde(crate = "rocket::serde")]
#[diesel(table_name = frameworks)]
pub struct NewFramework {
    pub language_id: i32,
    pub framework_name: String,
    pub display_order: Option<i32>,
}

#[derive(Deserialize, ToSchema)]
#[serde(crate = "rocket::serde")]
pub struct NewFrameworkRequest {
    pub framework_name: String,
    pub display_order: Option<i32>,
}

#[derive(AsChangeset, Deserialize, ToSchema)]
#[serde(crate = "rocket::serde")]
#[diesel(table_name = frameworks)]
pub struct UpdateFramework {
    pub framework_name: Option<String>,
    pub display_order: Option<i32>,
}
