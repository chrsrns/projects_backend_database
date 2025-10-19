use crate::schema::resumes;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use rocket::serde::{Deserialize, Serialize};
use std::cmp::{Eq, Ord, PartialEq, PartialOrd};

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
}