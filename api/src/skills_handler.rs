use application::resume::skills;
use domain::models::{NewSkillRequest, UpdateSkill};
use rocket::response::status::{Created, NoContent, NotFound};
use rocket::serde::json::Json;
use rocket::{delete as rocket_delete, get, post, put};

use crate::auth::{AuthSession, MaybeAuthSession};

#[get("/resume/<resume_id>/skills")]
pub fn list_skills_handler(
    resume_id: i32,
    maybe_auth: MaybeAuthSession,
) -> Result<String, NotFound<String>> {
    let user_id_value = maybe_auth.0.map(|a| a.user_id);
    skills::list_skills(resume_id, user_id_value)
}

#[post(
    "/resume/<resume_id>/skills",
    format = "application/json",
    data = "<payload>"
)]
pub fn create_skill_handler(
    auth: AuthSession,
    resume_id: i32,
    payload: Json<NewSkillRequest>,
) -> Result<Created<String>, rocket::response::status::Custom<String>> {
    skills::create_skill(auth.user_id, resume_id, payload)
}

#[put("/skills/<skill_id>", format = "application/json", data = "<payload>")]
pub fn update_skill_handler(
    auth: AuthSession,
    skill_id: i32,
    payload: Json<UpdateSkill>,
) -> Result<String, rocket::response::status::Custom<String>> {
    skills::update_skill(auth.user_id, skill_id, payload)
}

#[rocket_delete("/skills/<skill_id>")]
pub fn delete_skill_handler(
    auth: AuthSession,
    skill_id: i32,
) -> Result<NoContent, rocket::response::status::Custom<String>> {
    skills::delete_skill(auth.user_id, skill_id)
}
