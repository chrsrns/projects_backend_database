use application::resume::work_experiences;
use domain::models::{
    NewWorkExperienceKeyPointRequest, NewWorkExperienceRequest, UpdateWorkExperience,
    UpdateWorkExperienceKeyPoint,
};
use rocket::response::status::{Created, NoContent, NotFound};
use rocket::serde::json::Json;
use rocket::{delete as rocket_delete, get, post, put};

use crate::auth::{AuthSession, MaybeAuthSession};

#[get("/resume/<resume_id>/work_experiences")]
pub fn list_work_experiences_handler(
    resume_id: i32,
    maybe_auth: MaybeAuthSession,
) -> Result<String, NotFound<String>> {
    let user_id_value = maybe_auth.0.map(|a| a.user_id);
    work_experiences::list_work_experiences(resume_id, user_id_value)
}

#[post(
    "/resume/<resume_id>/work_experiences",
    format = "application/json",
    data = "<payload>"
)]
pub fn create_work_experience_handler(
    auth: AuthSession,
    resume_id: i32,
    payload: Json<NewWorkExperienceRequest>,
) -> Result<Created<String>, rocket::response::status::Custom<String>> {
    work_experiences::create_work_experience(auth.user_id, resume_id, payload)
}

#[put(
    "/work_experiences/<work_id>",
    format = "application/json",
    data = "<payload>"
)]
pub fn update_work_experience_handler(
    auth: AuthSession,
    work_id: i32,
    payload: Json<UpdateWorkExperience>,
) -> Result<String, rocket::response::status::Custom<String>> {
    work_experiences::update_work_experience(auth.user_id, work_id, payload)
}

#[rocket_delete("/work_experiences/<work_id>")]
pub fn delete_work_experience_handler(
    auth: AuthSession,
    work_id: i32,
) -> Result<NoContent, rocket::response::status::Custom<String>> {
    work_experiences::delete_work_experience(auth.user_id, work_id)
}

#[get("/resume/<resume_id>/work_experiences/<work_id>/key_points")]
pub fn list_work_experience_key_points_handler(
    resume_id: i32,
    work_id: i32,
    maybe_auth: MaybeAuthSession,
) -> Result<String, NotFound<String>> {
    let user_id_value = maybe_auth.0.map(|a| a.user_id);
    work_experiences::list_work_experience_key_points(resume_id, work_id, user_id_value)
}

#[post(
    "/resume/<resume_id>/work_experiences/<work_id>/key_points",
    format = "application/json",
    data = "<payload>"
)]
pub fn create_work_experience_key_point_handler(
    auth: AuthSession,
    resume_id: i32,
    work_id: i32,
    payload: Json<NewWorkExperienceKeyPointRequest>,
) -> Result<Created<String>, rocket::response::status::Custom<String>> {
    work_experiences::create_work_experience_key_point(auth.user_id, resume_id, work_id, payload)
}

#[put(
    "/work_experience_key_points/<key_point_id>",
    format = "application/json",
    data = "<payload>"
)]
pub fn update_work_experience_key_point_handler(
    auth: AuthSession,
    key_point_id: i32,
    payload: Json<UpdateWorkExperienceKeyPoint>,
) -> Result<String, rocket::response::status::Custom<String>> {
    work_experiences::update_work_experience_key_point(auth.user_id, key_point_id, payload)
}

#[rocket_delete("/work_experience_key_points/<key_point_id>")]
pub fn delete_work_experience_key_point_handler(
    auth: AuthSession,
    key_point_id: i32,
) -> Result<NoContent, rocket::response::status::Custom<String>> {
    work_experiences::delete_work_experience_key_point(auth.user_id, key_point_id)
}
