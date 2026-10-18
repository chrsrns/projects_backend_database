use application::resume::{create, delete, read, update};
use domain::models::{NewResumeRequest, ResumeListParams, ResumePage, ResumeView, UpdateResume};
use rocket::State;
use rocket::http::Status;
use rocket::response::status::{Custom, NoContent};
use rocket::serde::json::Json;
use rocket::{delete as rocket_delete, get, post, put};
use shared::response_models::Response;

use super::{CustomJsonResult, JsonResult, NoContentResult};
use crate::auth::{AuthSession, MaybeAuthSession};
use crate::error::ApiError;
use crate::query_scan::QueryScan;
use crate::realtime::Hub;
use uuid::Uuid;

#[utoipa::path(
    get,
    path = "/resumes",
    tag = "Resumes",
    params(
        ("limit" = Option<i64>, Query, description = "Page size (1-100, default 20)"),
        ("offset" = Option<i64>, Query, description = "Rows to skip (default 0)"),
        ("sort" = Option<String>, Query, description = "Sort field: id | name | updated_at"),
        ("order" = Option<String>, Query, description = "Sort order: asc | desc"),
        ("search" = Option<String>, Query, description = "AND-of-terms substring search")
    ),
    responses(
        (status = 200, description = "OK", body = Response<ResumePage>, content_type = "application/json"),
        (status = 400, description = "Bad Request", body = Response<String>, content_type = "application/json")
    )
)]
#[get("/resumes?<params..>")]
pub fn list_resumes_handler(
    maybe_auth: MaybeAuthSession,
    params: Result<ResumeListParams, rocket::form::Errors<'_>>,
    scan: QueryScan,
) -> JsonResult<ResumePage> {
    let user_id_value = maybe_auth.0.map(|a| a.user_id);
    let bad_request = || {
        ApiError(Custom(
            Status::BadRequest,
            Json(Response {
                body: "invalid query parameters".to_string(),
            }),
        ))
    };
    if !scan.is_clean() {
        return Err(bad_request());
    }
    let params = params.map_err(|_| bad_request())?;
    let page = read::list_resumes(user_id_value, params)?;
    Ok(Json(Response { body: page }))
}

#[utoipa::path(
    get,
    path = "/resume/{resume_id}",
    tag = "Resumes",
    params(
        ("resume_id" = Uuid, Path, description = "Resume id")
    ),
    responses(
        (status = 200, description = "OK", body = Response<ResumeView>, content_type = "application/json"),
        (status = 403, description = "Forbidden", body = Response<String>, content_type = "application/json"),
        (status = 404, description = "Not Found", body = Response<String>, content_type = "application/json")
    )
)]
#[get("/resume/<resume_id>")]
pub fn list_resume_handler(
    resume_id: Uuid,
    maybe_auth: MaybeAuthSession,
) -> JsonResult<ResumeView> {
    let user_id_value = maybe_auth.0.map(|a| a.user_id);
    let resume = read::list_resume(resume_id, user_id_value)?;
    Ok(Json(Response { body: resume }))
}

#[utoipa::path(
    post,
    path = "/new_resume",
    tag = "Resumes",
    security(("bearerAuth" = [])),
    request_body(content = NewResumeRequest, content_type = "application/json"),
    responses(
        (status = 201, description = "Created", body = Response<ResumeView>, content_type = "application/json"),
        (status = 401, description = "Unauthorized", body = Response<String>, content_type = "application/json"),
        (status = 409, description = "Conflict", body = Response<String>, content_type = "application/json")
    )
)]
#[post("/new_resume", format = "application/json", data = "<resume>")]
pub fn create_resume_handler(
    auth: AuthSession,
    hub: &State<Hub>,
    resume: Json<NewResumeRequest>,
) -> CustomJsonResult<ResumeView> {
    let resume = create::create_resume(auth.user_id, resume.into_inner(), hub.inner())?;
    Ok(Custom(
        rocket::http::Status::Created,
        Json(Response { body: resume }),
    ))
}

#[utoipa::path(
    put,
    path = "/resume/{resume_id}",
    tag = "Resumes",
    security(("bearerAuth" = [])),
    params(
        ("resume_id" = Uuid, Path, description = "Resume id")
    ),
    request_body(content = UpdateResume, content_type = "application/json"),
    responses(
        (status = 200, description = "OK", body = Response<ResumeView>, content_type = "application/json"),
        (status = 401, description = "Unauthorized", body = Response<String>, content_type = "application/json"),
        (status = 403, description = "Forbidden", body = Response<String>, content_type = "application/json"),
        (status = 404, description = "Not Found", body = Response<String>, content_type = "application/json")
    )
)]
#[put("/resume/<resume_id>", format = "application/json", data = "<resume>")]
pub fn update_resume_handler(
    auth: AuthSession,
    hub: &State<Hub>,
    resume_id: Uuid,
    resume: Json<UpdateResume>,
) -> JsonResult<ResumeView> {
    let updated = update::update_resume(auth.user_id, resume_id, resume.into_inner(), hub.inner())?;
    Ok(Json(Response { body: updated }))
}

#[utoipa::path(
    delete,
    path = "/resume/{resume_id}",
    tag = "Resumes",
    security(("bearerAuth" = [])),
    params(
        ("resume_id" = Uuid, Path, description = "Resume id")
    ),
    responses(
        (status = 204, description = "No Content"),
        (status = 401, description = "Unauthorized", body = Response<String>, content_type = "application/json"),
        (status = 403, description = "Forbidden", body = Response<String>, content_type = "application/json"),
        (status = 404, description = "Not Found", body = Response<String>, content_type = "application/json")
    )
)]
#[rocket_delete("/resume/<resume_id>")]
pub fn delete_resume_handler(
    auth: AuthSession,
    hub: &State<Hub>,
    resume_id: Uuid,
) -> NoContentResult {
    delete::delete_resume(auth.user_id, resume_id, hub.inner())?;
    Ok(NoContent)
}
