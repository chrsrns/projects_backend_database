use chrono::NaiveDateTime;
use domain::models::{Resume, User};
use rocket::serde::Serialize;

#[derive(Serialize)]
pub enum ResponseBody {
    Message(String),
    Resume(Resume),
    Resumes(Vec<Resume>),
    User(User),
    AuthToken(AuthTokenResponse),
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct AuthTokenResponse {
    pub token: String,
    pub expires_at: NaiveDateTime,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct Response {
    pub body: ResponseBody,
}
