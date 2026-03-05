use application::error::ApplicationError;
use application::resume::frameworks;
use domain::models::{Framework, NewFrameworkRequest, UpdateFramework};
use rocket::State;
use rocket::response::status::{Custom, NoContent};
use rocket::serde::json::Json;
use rocket::{delete as rocket_delete, get, post, put};
use shared::response_models::Response;

use crate::auth::{AuthSession, MaybeAuthSession};
use crate::realtime::{Hub, ResumeChangedAction};

#[utoipa::path(
    get,
    path = "/resume/{resume_id}/languages/{language_id}/frameworks",
    tag = "Frameworks",
    params(
        ("resume_id" = i32, Path, description = "Resume id"),
        ("language_id" = i32, Path, description = "Language id")
    ),
    responses(
        (status = 200, description = "OK", body = Response<Vec<Framework>>, content_type = "application/json"),
        (status = 401, description = "Unauthorized", body = Response<String>, content_type = "application/json"),
        (status = 403, description = "Forbidden", body = Response<String>, content_type = "application/json"),
        (status = 404, description = "Not Found", body = Response<String>, content_type = "application/json")
    )
)]
#[get("/resume/<resume_id>/languages/<language_id>/frameworks")]
pub fn list_frameworks_handler(
    resume_id: i32,
    language_id: i32,
    maybe_auth: MaybeAuthSession,
) -> Result<Json<Response<Vec<Framework>>>, Custom<Json<Response<String>>>> {
    let user_id_value = maybe_auth.0.map(|a| a.user_id);

    match frameworks::list_frameworks(resume_id, language_id, user_id_value) {
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
    path = "/resume/{resume_id}/languages/{language_id}/frameworks",
    tag = "Frameworks",
    security(("bearerAuth" = [])),
    params(
        ("resume_id" = i32, Path, description = "Resume id"),
        ("language_id" = i32, Path, description = "Language id")
    ),
    request_body(content = NewFrameworkRequest, content_type = "application/json"),
    responses(
        (status = 201, description = "Created", body = Response<Framework>, content_type = "application/json"),
        (status = 401, description = "Unauthorized", body = Response<String>, content_type = "application/json"),
        (status = 403, description = "Forbidden", body = Response<String>, content_type = "application/json"),
        (status = 404, description = "Not Found", body = Response<String>, content_type = "application/json")
    )
)]
#[post(
    "/resume/<resume_id>/languages/<language_id>/frameworks",
    format = "application/json",
    data = "<payload>"
)]
pub fn create_framework_handler(
    auth: AuthSession,
    hub: &State<Hub>,
    resume_id: i32,
    language_id: i32,
    payload: Json<NewFrameworkRequest>,
) -> Result<Custom<Json<Response<Framework>>>, Custom<Json<Response<String>>>> {
    match frameworks::create_framework(auth.user_id, resume_id, language_id, payload.into_inner()) {
        Ok(framework) => {
            hub.publish_resume_changed(resume_id, ResumeChangedAction::Updated);
            Ok(Custom(
                rocket::http::Status::Created,
                Json(Response { body: framework }),
            ))
        }
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
    path = "/frameworks/{framework_id}",
    tag = "Frameworks",
    security(("bearerAuth" = [])),
    params(
        ("framework_id" = i32, Path, description = "Framework id")
    ),
    request_body(content = UpdateFramework, content_type = "application/json"),
    responses(
        (status = 200, description = "OK", body = Response<Framework>, content_type = "application/json"),
        (status = 401, description = "Unauthorized", body = Response<String>, content_type = "application/json"),
        (status = 403, description = "Forbidden", body = Response<String>, content_type = "application/json"),
        (status = 404, description = "Not Found", body = Response<String>, content_type = "application/json")
    )
)]
#[put(
    "/frameworks/<framework_id>",
    format = "application/json",
    data = "<payload>"
)]
pub fn update_framework_handler(
    auth: AuthSession,
    hub: &State<Hub>,
    framework_id: i32,
    payload: Json<UpdateFramework>,
) -> Result<Json<Response<Framework>>, Custom<Json<Response<String>>>> {
    match frameworks::update_framework(auth.user_id, framework_id, payload.into_inner()) {
        Ok((framework, resume_id)) => {
            hub.publish_resume_changed(resume_id, ResumeChangedAction::Updated);
            Ok(Json(Response { body: framework }))
        }
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
    path = "/frameworks/{framework_id}",
    tag = "Frameworks",
    security(("bearerAuth" = [])),
    params(
        ("framework_id" = i32, Path, description = "Framework id")
    ),
    responses(
        (status = 204, description = "No Content"),
        (status = 401, description = "Unauthorized", body = Response<String>, content_type = "application/json"),
        (status = 403, description = "Forbidden", body = Response<String>, content_type = "application/json"),
        (status = 404, description = "Not Found", body = Response<String>, content_type = "application/json")
    )
)]
#[rocket_delete("/frameworks/<framework_id>")]
pub fn delete_framework_handler(
    auth: AuthSession,
    hub: &State<Hub>,
    framework_id: i32,
) -> Result<NoContent, Custom<Json<Response<String>>>> {
    match frameworks::delete_framework(auth.user_id, framework_id) {
        Ok(resume_id) => {
            hub.publish_resume_changed(resume_id, ResumeChangedAction::Updated);
            Ok(NoContent)
        }
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
