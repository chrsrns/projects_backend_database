use crate::schema::skills;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use rocket::serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Queryable, Serialize, ToSchema, Ord, Eq, PartialEq, PartialOrd)]
pub struct Skill {
    pub id: i32,
    pub skill_name: String,
    pub confidence_percentage: i32,
    pub display_order: Option<i32>,
    pub created_at: NaiveDateTime,
    pub resume_id: Uuid,
}

#[derive(Insertable, Deserialize, ToSchema)]
#[serde(crate = "rocket::serde")]
#[diesel(table_name = skills)]
pub struct NewSkill {
    pub resume_id: Uuid,
    pub skill_name: String,
    pub confidence_percentage: i32,
    pub display_order: Option<i32>,
}

#[derive(Deserialize, ToSchema)]
#[serde(crate = "rocket::serde")]
pub struct NewSkillRequest {
    pub skill_name: String,
    pub confidence_percentage: i32,
    pub display_order: Option<i32>,
}

#[derive(AsChangeset, Deserialize, ToSchema)]
#[serde(crate = "rocket::serde")]
#[diesel(table_name = skills)]
pub struct UpdateSkill {
    pub skill_name: Option<String>,
    pub confidence_percentage: Option<i32>,
    pub display_order: Option<i32>,
}
