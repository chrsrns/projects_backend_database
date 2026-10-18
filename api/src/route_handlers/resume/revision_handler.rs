use application::resume::revision::{read, restore};
use domain::models::{
    RestoreRequest, ResumeRevision, ResumeRevisionDiff, ResumeRevisionSummary, ResumeView,
};
use rocket::State;
use rocket::http::Status;
use rocket::response::status::Custom;
use rocket::serde::json::Json;
use rocket::{get, post};
use shared::response_models::Response;
use uuid::Uuid;

use super::JsonResult;
use crate::auth::AuthSession;
use crate::error::ApiError;
use crate::realtime::Hub;

#[utoipa::path(
    get,
    path = "/resume/{resume_id}/revisions",
    tag = "Resumes",
    security(("bearerAuth" = [])),
    params(
        ("resume_id" = Uuid, Path, description = "Resume ID")
    ),
    responses(
        (status = 200, description = "Revision timeline, newest first", body = Response<Vec<ResumeRevisionSummary>>, content_type = "application/json"),
        (status = 401, description = "Unauthorized", body = Response<String>, content_type = "application/json"),
        (status = 403, description = "Forbidden", body = Response<String>, content_type = "application/json"),
        (status = 404, description = "Not Found", body = Response<String>, content_type = "application/json")
    )
)]
#[get("/resume/<resume_id>/revisions")]
pub fn list_revisions_handler(
    auth: AuthSession,
    resume_id: Uuid,
) -> JsonResult<Vec<ResumeRevisionSummary>> {
    let revisions = read::list_revisions(resume_id, auth.user_id)?;
    Ok(Json(Response { body: revisions }))
}

#[utoipa::path(
    get,
    path = "/resume/{resume_id}/revisions/{revision}",
    tag = "Resumes",
    security(("bearerAuth" = [])),
    params(
        ("resume_id" = Uuid, Path, description = "Resume ID"),
        ("revision" = i32, Path, description = "Revision number")
    ),
    responses(
        (status = 200, description = "One revision with the state it recorded", body = Response<ResumeRevision>, content_type = "application/json"),
        (status = 401, description = "Unauthorized", body = Response<String>, content_type = "application/json"),
        (status = 403, description = "Forbidden", body = Response<String>, content_type = "application/json"),
        (status = 404, description = "Not Found", body = Response<String>, content_type = "application/json")
    )
)]
#[get("/resume/<resume_id>/revisions/<revision>")]
pub fn load_revision_handler(
    auth: AuthSession,
    resume_id: Uuid,
    revision: i32,
) -> JsonResult<ResumeRevision> {
    let revision = read::load_revision(resume_id, revision, auth.user_id)?;
    Ok(Json(Response { body: revision }))
}

#[utoipa::path(
    get,
    path = "/resume/{resume_id}/revisions/{revision}/diff",
    tag = "Resumes",
    security(("bearerAuth" = [])),
    params(
        ("resume_id" = Uuid, Path, description = "Resume ID"),
        ("revision" = i32, Path, description = "Revision number"),
        ("unified" = Option<String>, Query, description = "Set to true to include a unified text diff")
    ),
    responses(
        (status = 200, description = "What the edit changed", body = Response<ResumeRevisionDiff>, content_type = "application/json"),
        (status = 400, description = "Bad Request", body = Response<String>, content_type = "application/json"),
        (status = 401, description = "Unauthorized", body = Response<String>, content_type = "application/json"),
        (status = 403, description = "Forbidden", body = Response<String>, content_type = "application/json"),
        (status = 404, description = "Not Found", body = Response<String>, content_type = "application/json")
    )
)]
#[get("/resume/<resume_id>/revisions/<revision>/diff?<unified>")]
pub fn revision_diff_handler(
    auth: AuthSession,
    resume_id: Uuid,
    revision: i32,
    unified: Option<String>,
) -> JsonResult<ResumeRevisionDiff> {
    let include_unified = match unified.as_deref() {
        None => false,
        Some("true") => true,
        Some(other) => {
            return Err(ApiError(Custom(
                Status::BadRequest,
                Json(Response {
                    body: format!("unified must be true, got {}", other),
                }),
            )));
        }
    };

    let diff = read::revision_diff(resume_id, revision, auth.user_id, include_unified)?;
    Ok(Json(Response { body: diff }))
}

#[utoipa::path(
    post,
    path = "/resume/{resume_id}/revisions/{revision}/restore",
    tag = "Resumes",
    security(("bearerAuth" = [])),
    params(
        ("resume_id" = Uuid, Path, description = "Resume ID"),
        ("revision" = i32, Path, description = "Revision number to restore")
    ),
    request_body(content = RestoreRequest, content_type = "application/json"),
    responses(
        (status = 200, description = "Restored resume", body = Response<ResumeView>, content_type = "application/json"),
        (status = 400, description = "Bad Request", body = Response<String>, content_type = "application/json"),
        (status = 401, description = "Unauthorized", body = Response<String>, content_type = "application/json"),
        (status = 403, description = "Forbidden", body = Response<String>, content_type = "application/json"),
        (status = 404, description = "Not Found", body = Response<String>, content_type = "application/json"),
        (status = 409, description = "Conflict", body = Response<String>, content_type = "application/json")
    )
)]
#[post(
    "/resume/<resume_id>/revisions/<revision>/restore",
    format = "application/json",
    data = "<request>"
)]
pub fn restore_revision_handler(
    auth: AuthSession,
    hub: &State<Hub>,
    resume_id: Uuid,
    revision: i32,
    request: Json<RestoreRequest>,
) -> JsonResult<ResumeView> {
    let restored = restore::restore_revision(
        auth.user_id,
        resume_id,
        revision,
        request.expected_revision,
        hub.inner(),
    )?;

    Ok(Json(Response { body: restored }))
}
