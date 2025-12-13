use crate::schema::{sessions, users};
use chrono::NaiveDateTime;
use diesel::prelude::*;
use rocket::serde::{Deserialize, Serialize};
use std::cmp::{Eq, Ord, PartialEq, PartialOrd};
use uuid::Uuid;

// ==================== Auth ====================

#[derive(Queryable, Serialize, Ord, Eq, PartialEq, PartialOrd)]
pub struct User {
    pub id: i32,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable, Deserialize)]
#[serde(crate = "rocket::serde")]
#[diesel(table_name = users)]
pub struct NewUser {
    pub email: String,
    pub password_hash: String,
}

#[derive(Queryable, Serialize, Ord, Eq, PartialEq, PartialOrd)]
pub struct Session {
    pub id: Uuid,
    pub user_id: i32,
    pub expires_at: NaiveDateTime,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable, Deserialize)]
#[serde(crate = "rocket::serde")]
#[diesel(table_name = sessions)]
pub struct NewSession {
    pub user_id: i32,
    pub expires_at: NaiveDateTime,
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct AuthRegisterRequest {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct AuthLoginRequest {
    pub email: String,
    pub password: String,
}
