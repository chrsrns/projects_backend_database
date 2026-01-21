use application::resume::languages;
use domain::models::{NewLanguageRequest, UpdateLanguage};
use rocket::response::status::{Created, NoContent, NotFound};
use rocket::serde::json::Json;
use rocket::{delete as rocket_delete, get, post, put};

use crate::auth::{AuthSession, MaybeAuthSession};

#[get("/resume/<resume_id>/languages")]
pub fn list_languages_handler(
    resume_id: i32,
    maybe_auth: MaybeAuthSession,
) -> Result<String, NotFound<String>> {
    let user_id_value = maybe_auth.0.map(|a| a.user_id);
    languages::list_languages(resume_id, user_id_value)
}

#[post(
    "/resume/<resume_id>/languages",
    format = "application/json",
    data = "<payload>"
)]
pub fn create_language_handler(
    auth: AuthSession,
    resume_id: i32,
    payload: Json<NewLanguageRequest>,
) -> Result<Created<String>, rocket::response::status::Custom<String>> {
    languages::create_language(auth.user_id, resume_id, payload)
}

#[put(
    "/languages/<language_id>",
    format = "application/json",
    data = "<payload>"
)]
pub fn update_language_handler(
    auth: AuthSession,
    language_id: i32,
    payload: Json<UpdateLanguage>,
) -> Result<String, rocket::response::status::Custom<String>> {
    languages::update_language(auth.user_id, language_id, payload)
}

#[rocket_delete("/languages/<language_id>")]
pub fn delete_language_handler(
    auth: AuthSession,
    language_id: i32,
) -> Result<NoContent, rocket::response::status::Custom<String>> {
    languages::delete_language(auth.user_id, language_id)
}
