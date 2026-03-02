use application::error::ApplicationError;
use application::resume::skills;
use domain::models::{NewSkillRequest, Skill, UpdateSkill};
use rocket::response::status::{Custom, NoContent};
use rocket::serde::json::Json;
use rocket::{delete as rocket_delete, get, post, put};
use shared::response_models::Response;

use crate::auth::{AuthSession, MaybeAuthSession};

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
pub fn list_skills_handler(
    resume_id: i32,
    maybe_auth: MaybeAuthSession,
) -> Result<Json<Response<Vec<Skill>>>, Custom<Json<Response<String>>>> {
    let user_id_value = maybe_auth.0.map(|a| a.user_id);

    match skills::list_skills(resume_id, user_id_value) {
        Ok(items) => Ok(Json(Response { body: items })),
        Err(ApplicationError::NotFound(msg)) => Err(Custom(
            rocket::http::Status::NotFound,
            Json(Response { body: msg }),
        )),
        Err(ApplicationError::Forbidden) => Err(Custom(
            rocket::http::Status::Forbidden,
            Json(Response {
                body: "Forbidden".to_string(),
            }),
        )),
        Err(ApplicationError::Unauthorized) => Err(Custom(
            rocket::http::Status::Unauthorized,
            Json(Response {
                body: "Unauthorized".to_string(),
            }),
        )),
        Err(ApplicationError::Conflict(msg)) => Err(Custom(
            rocket::http::Status::Conflict,
            Json(Response { body: msg }),
        )),
        Err(ApplicationError::BadRequest(msg)) => Err(Custom(
            rocket::http::Status::BadRequest,
            Json(Response { body: msg }),
        )),
        Err(ApplicationError::Internal(msg)) => Err(Custom(
            rocket::http::Status::InternalServerError,
            Json(Response { body: msg }),
        )),
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
    resume_id: i32,
    payload: Json<NewSkillRequest>,
) -> Result<Custom<Json<Response<Skill>>>, Custom<Json<Response<String>>>> {
    match skills::create_skill(auth.user_id, resume_id, payload.into_inner()) {
        Ok(skill) => Ok(Custom(
            rocket::http::Status::Created,
            Json(Response { body: skill }),
        )),
        Err(ApplicationError::NotFound(msg)) => Err(Custom(
            rocket::http::Status::NotFound,
            Json(Response { body: msg }),
        )),
        Err(ApplicationError::Forbidden) => Err(Custom(
            rocket::http::Status::Forbidden,
            Json(Response {
                body: "Forbidden".to_string(),
            }),
        )),
        Err(ApplicationError::Unauthorized) => Err(Custom(
            rocket::http::Status::Unauthorized,
            Json(Response {
                body: "Unauthorized".to_string(),
            }),
        )),
        Err(ApplicationError::Conflict(msg)) => Err(Custom(
            rocket::http::Status::Conflict,
            Json(Response { body: msg }),
        )),
        Err(ApplicationError::BadRequest(msg)) => Err(Custom(
            rocket::http::Status::BadRequest,
            Json(Response { body: msg }),
        )),
        Err(ApplicationError::Internal(msg)) => Err(Custom(
            rocket::http::Status::InternalServerError,
            Json(Response { body: msg }),
        )),
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
    skill_id: i32,
    payload: Json<UpdateSkill>,
) -> Result<Json<Response<Skill>>, Custom<Json<Response<String>>>> {
    match skills::update_skill(auth.user_id, skill_id, payload.into_inner()) {
        Ok(skill) => Ok(Json(Response { body: skill })),
        Err(ApplicationError::NotFound(msg)) => Err(Custom(
            rocket::http::Status::NotFound,
            Json(Response { body: msg }),
        )),
        Err(ApplicationError::Forbidden) => Err(Custom(
            rocket::http::Status::Forbidden,
            Json(Response {
                body: "Forbidden".to_string(),
            }),
        )),
        Err(ApplicationError::Unauthorized) => Err(Custom(
            rocket::http::Status::Unauthorized,
            Json(Response {
                body: "Unauthorized".to_string(),
            }),
        )),
        Err(ApplicationError::Conflict(msg)) => Err(Custom(
            rocket::http::Status::Conflict,
            Json(Response { body: msg }),
        )),
        Err(ApplicationError::BadRequest(msg)) => Err(Custom(
            rocket::http::Status::BadRequest,
            Json(Response { body: msg }),
        )),
        Err(ApplicationError::Internal(msg)) => Err(Custom(
            rocket::http::Status::InternalServerError,
            Json(Response { body: msg }),
        )),
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
pub fn delete_skill_handler(
    auth: AuthSession,
    skill_id: i32,
) -> Result<NoContent, Custom<Json<Response<String>>>> {
    match skills::delete_skill(auth.user_id, skill_id) {
        Ok(()) => Ok(NoContent),
        Err(ApplicationError::NotFound(msg)) => Err(Custom(
            rocket::http::Status::NotFound,
            Json(Response { body: msg }),
        )),
        Err(ApplicationError::Forbidden) => Err(Custom(
            rocket::http::Status::Forbidden,
            Json(Response {
                body: "Forbidden".to_string(),
            }),
        )),
        Err(ApplicationError::Unauthorized) => Err(Custom(
            rocket::http::Status::Unauthorized,
            Json(Response {
                body: "Unauthorized".to_string(),
            }),
        )),
        Err(ApplicationError::Conflict(msg)) => Err(Custom(
            rocket::http::Status::Conflict,
            Json(Response { body: msg }),
        )),
        Err(ApplicationError::BadRequest(msg)) => Err(Custom(
            rocket::http::Status::BadRequest,
            Json(Response { body: msg }),
        )),
        Err(ApplicationError::Internal(msg)) => Err(Custom(
            rocket::http::Status::InternalServerError,
            Json(Response { body: msg }),
        )),
    }
}
