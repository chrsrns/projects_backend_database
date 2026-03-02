use chrono::NaiveDateTime;
use rocket::serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
#[serde(crate = "rocket::serde")]
pub struct AuthTokenResponse {
    pub token: String,
    pub expires_at: NaiveDateTime,
}

#[derive(Serialize, ToSchema)]
#[serde(crate = "rocket::serde")]
pub struct Response<T> {
    pub body: T,
}
