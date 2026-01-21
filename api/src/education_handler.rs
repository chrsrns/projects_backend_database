use application::resume::education;
use domain::models::{
    NewEducationKeyPointRequest, NewEducationRequest, UpdateEducation, UpdateEducationKeyPoint,
};
use rocket::response::status::{Created, NoContent, NotFound};
use rocket::serde::json::Json;
use rocket::{delete as rocket_delete, get, post, put};

use crate::auth::{AuthSession, MaybeAuthSession};

#[get("/resume/<resume_id>/education")]
pub fn list_educations_handler(
    resume_id: i32,
    maybe_auth: MaybeAuthSession,
) -> Result<String, NotFound<String>> {
    let user_id_value = maybe_auth.0.map(|a| a.user_id);
    education::list_educations(resume_id, user_id_value)
}

#[post(
    "/resume/<resume_id>/education",
    format = "application/json",
    data = "<payload>"
)]
pub fn create_education_handler(
    auth: AuthSession,
    resume_id: i32,
    payload: Json<NewEducationRequest>,
) -> Result<Created<String>, rocket::response::status::Custom<String>> {
    education::create_education(auth.user_id, resume_id, payload)
}

#[put(
    "/education/<education_id>",
    format = "application/json",
    data = "<payload>"
)]
pub fn update_education_handler(
    auth: AuthSession,
    education_id: i32,
    payload: Json<UpdateEducation>,
) -> Result<String, rocket::response::status::Custom<String>> {
    education::update_education(auth.user_id, education_id, payload)
}

#[rocket_delete("/education/<education_id>")]
pub fn delete_education_handler(
    auth: AuthSession,
    education_id: i32,
) -> Result<NoContent, rocket::response::status::Custom<String>> {
    education::delete_education(auth.user_id, education_id)
}

#[get("/resume/<resume_id>/education/<education_id>/key_points")]
pub fn list_education_key_points_handler(
    resume_id: i32,
    education_id: i32,
    maybe_auth: MaybeAuthSession,
) -> Result<String, NotFound<String>> {
    let user_id_value = maybe_auth.0.map(|a| a.user_id);
    education::list_education_key_points(resume_id, education_id, user_id_value)
}

#[post(
    "/resume/<resume_id>/education/<education_id>/key_points",
    format = "application/json",
    data = "<payload>"
)]
pub fn create_education_key_point_handler(
    auth: AuthSession,
    resume_id: i32,
    education_id: i32,
    payload: Json<NewEducationKeyPointRequest>,
) -> Result<Created<String>, rocket::response::status::Custom<String>> {
    education::create_education_key_point(auth.user_id, resume_id, education_id, payload)
}

#[put(
    "/education_key_points/<key_point_id>",
    format = "application/json",
    data = "<payload>"
)]
pub fn update_education_key_point_handler(
    auth: AuthSession,
    key_point_id: i32,
    payload: Json<UpdateEducationKeyPoint>,
) -> Result<String, rocket::response::status::Custom<String>> {
    education::update_education_key_point(auth.user_id, key_point_id, payload)
}

#[rocket_delete("/education_key_points/<key_point_id>")]
pub fn delete_education_key_point_handler(
    auth: AuthSession,
    key_point_id: i32,
) -> Result<NoContent, rocket::response::status::Custom<String>> {
    education::delete_education_key_point(auth.user_id, key_point_id)
}
