use application::error::ApplicationError;
use application::resume::languages;
use domain::models::{Language, NewLanguageRequest, UpdateLanguage};
use rocket::State;
use rocket::response::status::{Custom, NoContent};
use rocket::serde::json::Json;
use rocket::{delete as rocket_delete, get, post, put};
use shared::response_models::Response;

use crate::auth::{AuthSession, MaybeAuthSession};
use crate::realtime::{Hub, ResumeChangedAction};

#[utoipa::path(
    get,
    path = "/resume/{resume_id}/languages",
    tag = "Languages",
    params(
        ("resume_id" = i32, Path, description = "Resume id")
    ),
    responses(
        (status = 200, description = "OK", body = Response<Vec<Language>>, content_type = "application/json"),
        (status = 401, description = "Unauthorized", body = Response<String>, content_type = "application/json"),
        (status = 403, description = "Forbidden", body = Response<String>, content_type = "application/json"),
        (status = 404, description = "Not Found", body = Response<String>, content_type = "application/json")
    )
)]
#[get("/resume/<resume_id>/languages")]
pub fn list_languages_handler(
    resume_id: i32,
    maybe_auth: MaybeAuthSession,
) -> Result<Json<Response<Vec<Language>>>, Custom<Json<Response<String>>>> {
    let user_id_value = maybe_auth.0.map(|a| a.user_id);

    match languages::list_languages(resume_id, user_id_value) {
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
    path = "/resume/{resume_id}/languages",
    tag = "Languages",
    security(("bearerAuth" = [])),
    params(
        ("resume_id" = i32, Path, description = "Resume id")
    ),
    request_body(content = NewLanguageRequest, content_type = "application/json"),
    responses(
        (status = 201, description = "Created", body = Response<Language>, content_type = "application/json"),
        (status = 401, description = "Unauthorized", body = Response<String>, content_type = "application/json"),
        (status = 403, description = "Forbidden", body = Response<String>, content_type = "application/json"),
        (status = 404, description = "Not Found", body = Response<String>, content_type = "application/json")
    )
)]
#[post(
    "/resume/<resume_id>/languages",
    format = "application/json",
    data = "<payload>"
)]
pub fn create_language_handler(
    auth: AuthSession,
    hub: &State<Hub>,
    resume_id: i32,
    payload: Json<NewLanguageRequest>,
) -> Result<Custom<Json<Response<Language>>>, Custom<Json<Response<String>>>> {
    match languages::create_language(auth.user_id, resume_id, payload.into_inner()) {
        Ok(language) => {
            hub.publish_resume_changed(
                resume_id,
                ResumeChangedAction::Updated(crate::realtime::SectionType::Languages),
            );
            Ok(Custom(
                rocket::http::Status::Created,
                Json(Response { body: language }),
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
    path = "/languages/{language_id}",
    tag = "Languages",
    security(("bearerAuth" = [])),
    params(
        ("language_id" = i32, Path, description = "Language id")
    ),
    request_body(content = UpdateLanguage, content_type = "application/json"),
    responses(
        (status = 200, description = "OK", body = Response<Language>, content_type = "application/json"),
        (status = 401, description = "Unauthorized", body = Response<String>, content_type = "application/json"),
        (status = 403, description = "Forbidden", body = Response<String>, content_type = "application/json"),
        (status = 404, description = "Not Found", body = Response<String>, content_type = "application/json")
    )
)]
#[put(
    "/languages/<language_id>",
    format = "application/json",
    data = "<payload>"
)]
pub fn update_language_handler(
    auth: AuthSession,
    hub: &State<Hub>,
    language_id: i32,
    payload: Json<UpdateLanguage>,
) -> Result<Json<Response<Language>>, Custom<Json<Response<String>>>> {
    match languages::update_language(auth.user_id, language_id, payload.into_inner()) {
        Ok(language) => {
            hub.publish_resume_changed(
                language.resume_id,
                ResumeChangedAction::Updated(crate::realtime::SectionType::Languages),
            );
            Ok(Json(Response { body: language }))
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
    path = "/languages/{language_id}",
    tag = "Languages",
    security(("bearerAuth" = [])),
    params(
        ("language_id" = i32, Path, description = "Language id")
    ),
    responses(
        (status = 204, description = "No Content"),
        (status = 401, description = "Unauthorized", body = Response<String>, content_type = "application/json"),
        (status = 403, description = "Forbidden", body = Response<String>, content_type = "application/json"),
        (status = 404, description = "Not Found", body = Response<String>, content_type = "application/json")
    )
)]
#[rocket_delete("/languages/<language_id>")]
pub fn delete_language_handler(
    auth: AuthSession,
    hub: &State<Hub>,
    language_id: i32,
) -> Result<NoContent, Custom<Json<Response<String>>>> {
    match languages::delete_language(auth.user_id, language_id) {
        Ok(resume_id) => {
            hub.publish_resume_changed(
                resume_id,
                ResumeChangedAction::Updated(crate::realtime::SectionType::Languages),
            );
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
