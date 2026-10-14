use application::resume::languages;
use domain::models::{Language, NewLanguageRequest, UpdateLanguage};
use rocket::State;
use rocket::response::status::{Custom, NoContent};
use rocket::serde::json::Json;
use rocket::{delete as rocket_delete, get, post, put};
use shared::response_models::Response;

use super::{CustomJsonResult, JsonResult, NoContentResult};
use crate::auth::{AuthSession, MaybeAuthSession};
use crate::error::ApiError;
use crate::realtime::Hub;

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
) -> JsonResult<Vec<Language>> {
    let user_id_value = maybe_auth.0.map(|a| a.user_id);
    match languages::list_languages(resume_id, user_id_value) {
        Ok(items) => Ok(Json(Response { body: items })),
        Err(err) => Err(ApiError::from(err)),
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
) -> CustomJsonResult<Language> {
    match languages::create_language(auth.user_id, resume_id, payload.into_inner(), hub.inner()) {
        Ok(language) => Ok(Custom(
            rocket::http::Status::Created,
            Json(Response { body: language }),
        )),
        Err(err) => Err(ApiError::from(err)),
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
) -> JsonResult<Language> {
    match languages::update_language(auth.user_id, language_id, payload.into_inner(), hub.inner()) {
        Ok(language) => Ok(Json(Response { body: language })),
        Err(err) => Err(ApiError::from(err)),
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
) -> NoContentResult {
    match languages::delete_language(auth.user_id, language_id, hub.inner()) {
        Ok(()) => Ok(NoContent),
        Err(err) => Err(ApiError::from(err)),
    }
}
