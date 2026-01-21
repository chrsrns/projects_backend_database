use application::resume::frameworks;
use domain::models::{NewFrameworkRequest, UpdateFramework};
use rocket::response::status::{Created, NoContent, NotFound};
use rocket::serde::json::Json;
use rocket::{delete as rocket_delete, get, post, put};

use crate::auth::{AuthSession, MaybeAuthSession};

#[get("/resume/<resume_id>/languages/<language_id>/frameworks")]
pub fn list_frameworks_handler(
    resume_id: i32,
    language_id: i32,
    maybe_auth: MaybeAuthSession,
) -> Result<String, NotFound<String>> {
    let user_id_value = maybe_auth.0.map(|a| a.user_id);
    frameworks::list_frameworks(resume_id, language_id, user_id_value)
}

#[post(
    "/resume/<resume_id>/languages/<language_id>/frameworks",
    format = "application/json",
    data = "<payload>"
)]
pub fn create_framework_handler(
    auth: AuthSession,
    resume_id: i32,
    language_id: i32,
    payload: Json<NewFrameworkRequest>,
) -> Result<Created<String>, rocket::response::status::Custom<String>> {
    frameworks::create_framework(auth.user_id, resume_id, language_id, payload)
}

#[put(
    "/frameworks/<framework_id>",
    format = "application/json",
    data = "<payload>"
)]
pub fn update_framework_handler(
    auth: AuthSession,
    framework_id: i32,
    payload: Json<UpdateFramework>,
) -> Result<String, rocket::response::status::Custom<String>> {
    frameworks::update_framework(auth.user_id, framework_id, payload)
}

#[rocket_delete("/frameworks/<framework_id>")]
pub fn delete_framework_handler(
    auth: AuthSession,
    framework_id: i32,
) -> Result<NoContent, rocket::response::status::Custom<String>> {
    frameworks::delete_framework(auth.user_id, framework_id)
}
