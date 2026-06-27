use application::resume::skills;
use domain::models::{NewSkillRequest, Skill, UpdateSkill};
use rocket::State;
use rocket::response::status::{Custom, NoContent};
use rocket::serde::json::Json;
use rocket::{delete as rocket_delete, get, post, put};
use shared::response_models::Response;

use super::{CustomJsonResult, JsonResult, NoContentResult};
use crate::auth::{AuthSession, MaybeAuthSession};
use crate::error::map_application_error;
use crate::realtime::{Hub, ResumeChangedAction};

#[utoipa::path(
    get,
    path = "/resume/{resume_id}/skills",
    tag = "Skills",
    params(
        ("resume_id" = i32, Path, description = "Resume id")
    ),
    responses(
        (status = 200, description = "OK", body = Response<Vec<Skill>>, content_type = "application/json"),
        (status = 401, description = "Unauthorized", body = Response<String>, content_type = "application/json"),
        (status = 403, description = "Forbidden", body = Response<String>, content_type = "application/json"),
        (status = 404, description = "Not Found", body = Response<String>, content_type = "application/json")
    )
)]
#[get("/resume/<resume_id>/skills")]
pub fn list_skills_handler(resume_id: i32, maybe_auth: MaybeAuthSession) -> JsonResult<Vec<Skill>> {
    let user_id_value = maybe_auth.0.map(|a| a.user_id);
    match skills::list_skills(resume_id, user_id_value) {
        Ok(items) => Ok(Json(Response { body: items })),
        Err(err) => Err(map_application_error(err)),
    }
}

#[utoipa::path(
    post,
    path = "/resume/{resume_id}/skills",
    tag = "Skills",
    security(("bearerAuth" = [])),
    params(
        ("resume_id" = i32, Path, description = "Resume id")
    ),
    request_body(content = NewSkillRequest, content_type = "application/json"),
    responses(
        (status = 201, description = "Created", body = Response<Skill>, content_type = "application/json"),
        (status = 401, description = "Unauthorized", body = Response<String>, content_type = "application/json"),
        (status = 403, description = "Forbidden", body = Response<String>, content_type = "application/json"),
        (status = 404, description = "Not Found", body = Response<String>, content_type = "application/json")
    )
)]
#[post(
    "/resume/<resume_id>/skills",
    format = "application/json",
    data = "<payload>"
)]
pub fn create_skill_handler(
    auth: AuthSession,
    hub: &State<Hub>,
    resume_id: i32,
    payload: Json<NewSkillRequest>,
) -> CustomJsonResult<Skill> {
    match skills::create_skill(auth.user_id, resume_id, payload.into_inner()) {
        Ok(skill) => {
            hub.publish_resume_changed(
                resume_id,
                ResumeChangedAction::Updated(crate::realtime::SectionType::Skills),
            );
            Ok(Custom(
                rocket::http::Status::Created,
                Json(Response { body: skill }),
            ))
        }
        Err(err) => Err(map_application_error(err)),
    }
}

#[utoipa::path(
    put,
    path = "/skills/{skill_id}",
    tag = "Skills",
    security(("bearerAuth" = [])),
    params(
        ("skill_id" = i32, Path, description = "Skill id")
    ),
    request_body(content = UpdateSkill, content_type = "application/json"),
    responses(
        (status = 200, description = "OK", body = Response<Skill>, content_type = "application/json"),
        (status = 401, description = "Unauthorized", body = Response<String>, content_type = "application/json"),
        (status = 403, description = "Forbidden", body = Response<String>, content_type = "application/json"),
        (status = 404, description = "Not Found", body = Response<String>, content_type = "application/json")
    )
)]
#[put("/skills/<skill_id>", format = "application/json", data = "<payload>")]
pub fn update_skill_handler(
    auth: AuthSession,
    hub: &State<Hub>,
    skill_id: i32,
    payload: Json<UpdateSkill>,
) -> JsonResult<Skill> {
    match skills::update_skill(auth.user_id, skill_id, payload.into_inner()) {
        Ok(skill) => {
            hub.publish_resume_changed(
                skill.resume_id,
                ResumeChangedAction::Updated(crate::realtime::SectionType::Skills),
            );
            Ok(Json(Response { body: skill }))
        }
        Err(err) => Err(map_application_error(err)),
    }
}

#[utoipa::path(
    delete,
    path = "/skills/{skill_id}",
    tag = "Skills",
    security(("bearerAuth" = [])),
    params(
        ("skill_id" = i32, Path, description = "Skill id")
    ),
    responses(
        (status = 204, description = "No Content"),
        (status = 401, description = "Unauthorized", body = Response<String>, content_type = "application/json"),
        (status = 403, description = "Forbidden", body = Response<String>, content_type = "application/json"),
        (status = 404, description = "Not Found", body = Response<String>, content_type = "application/json")
    )
)]
#[rocket_delete("/skills/<skill_id>")]
pub fn delete_skill_handler(auth: AuthSession, hub: &State<Hub>, skill_id: i32) -> NoContentResult {
    match skills::delete_skill(auth.user_id, skill_id) {
        Ok(resume_id) => {
            hub.publish_resume_changed(
                resume_id,
                ResumeChangedAction::Updated(crate::realtime::SectionType::Skills),
            );
            Ok(NoContent)
        }
        Err(err) => Err(map_application_error(err)),
    }
}
