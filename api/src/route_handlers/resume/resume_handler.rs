use application::resume::{create, delete, read, update};
use domain::models::{NewResumeRequest, Resume, UpdateResume};
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
    path = "/resumes",
    tag = "Resumes",
    responses(
        (status = 200, description = "OK", body = Response<Vec<Resume>>, content_type = "application/json")
    )
)]
#[get("/resumes")]
pub fn list_resumes_handler(maybe_auth: MaybeAuthSession) -> JsonResult<Vec<Resume>> {
    let user_id_value = maybe_auth.0.map(|a| a.user_id);
    match read::list_resumes(user_id_value) {
        Ok(resumes) => Ok(Json(Response { body: resumes })),
        Err(err) => Err(map_application_error(err)),
    }
}

#[utoipa::path(
    get,
    path = "/resume/{resume_id}",
    tag = "Resumes",
    params(
        ("resume_id" = i32, Path, description = "Resume id")
    ),
    responses(
        (status = 200, description = "OK", body = Response<Resume>, content_type = "application/json"),
        (status = 403, description = "Forbidden", body = Response<String>, content_type = "application/json"),
        (status = 404, description = "Not Found", body = Response<String>, content_type = "application/json")
    )
)]
#[get("/resume/<resume_id>")]
pub fn list_resume_handler(resume_id: i32, maybe_auth: MaybeAuthSession) -> JsonResult<Resume> {
    let user_id_value = maybe_auth.0.map(|a| a.user_id);
    match read::list_resume(resume_id, user_id_value) {
        Ok(resume) => Ok(Json(Response { body: resume })),
        Err(err) => Err(map_application_error(err)),
    }
}

#[utoipa::path(
    post,
    path = "/new_resume",
    tag = "Resumes",
    security(("bearerAuth" = [])),
    request_body(content = NewResumeRequest, content_type = "application/json"),
    responses(
        (status = 201, description = "Created", body = Response<Resume>, content_type = "application/json"),
        (status = 401, description = "Unauthorized", body = Response<String>, content_type = "application/json"),
        (status = 409, description = "Conflict", body = Response<String>, content_type = "application/json")
    )
)]
#[post("/new_resume", format = "application/json", data = "<resume>")]
pub fn create_resume_handler(
    auth: AuthSession,
    hub: &State<Hub>,
    resume: Json<NewResumeRequest>,
) -> CustomJsonResult<Resume> {
    match create::create_resume(auth.user_id, resume.into_inner()) {
        Ok(resume) => {
            hub.publish_resume_changed(resume.id, ResumeChangedAction::Created);
            Ok(Custom(
                rocket::http::Status::Created,
                Json(Response { body: resume }),
            ))
        }
        Err(err) => Err(map_application_error(err)),
    }
}

#[utoipa::path(
    put,
    path = "/resume/{resume_id}",
    tag = "Resumes",
    security(("bearerAuth" = [])),
    params(
        ("resume_id" = i32, Path, description = "Resume id")
    ),
    request_body(content = UpdateResume, content_type = "application/json"),
    responses(
        (status = 200, description = "OK", body = Response<Resume>, content_type = "application/json"),
        (status = 401, description = "Unauthorized", body = Response<String>, content_type = "application/json"),
        (status = 403, description = "Forbidden", body = Response<String>, content_type = "application/json"),
        (status = 404, description = "Not Found", body = Response<String>, content_type = "application/json")
    )
)]
#[put("/resume/<resume_id>", format = "application/json", data = "<resume>")]
pub fn update_resume_handler(
    auth: AuthSession,
    hub: &State<Hub>,
    resume_id: i32,
    resume: Json<UpdateResume>,
) -> JsonResult<Resume> {
    match update::update_resume(auth.user_id, resume_id, resume.into_inner()) {
        Ok(updated) => {
            hub.publish_resume_changed(
                updated.id,
                ResumeChangedAction::Updated(crate::realtime::SectionType::PersonalInfo),
            );
            Ok(Json(Response { body: updated }))
        }
        Err(err) => Err(map_application_error(err)),
    }
}

#[utoipa::path(
    delete,
    path = "/resume/{resume_id}",
    tag = "Resumes",
    security(("bearerAuth" = [])),
    params(
        ("resume_id" = i32, Path, description = "Resume id")
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
    resume_id: i32,
) -> NoContentResult {
    match delete::delete_resume(auth.user_id, resume_id) {
        Ok(()) => {
            hub.publish_resume_changed(resume_id, ResumeChangedAction::Deleted);
            Ok(NoContent)
        }
        Err(err) => Err(map_application_error(err)),
    }
}
