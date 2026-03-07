use application::error::ApplicationError;
use application::resume::{create, delete, read, update};
use domain::models::{NewResumeRequest, Resume, UpdateResume};
use rocket::State;
use rocket::response::status::{Conflict, Custom, NoContent};
use rocket::serde::json::Json;
use rocket::{delete as rocket_delete, get, post, put};
use shared::response_models::Response;

use crate::auth::{AuthSession, MaybeAuthSession};
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
pub fn list_resumes_handler(
    maybe_auth: MaybeAuthSession,
) -> Result<Json<Response<Vec<Resume>>>, Custom<Json<Response<String>>>> {
    let user_id_value = maybe_auth.0.map(|a| a.user_id);

    let resumes: Vec<Resume> = read::list_resumes(user_id_value).map_err(|err| {
        let msg = match err {
            ApplicationError::Internal(msg) => msg,
            other => format!("{:?}", other),
        };
        Custom(
            rocket::http::Status::InternalServerError,
            Json(Response { body: msg }),
        )
    })?;

    Ok(Json(Response { body: resumes }))
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
pub fn list_resume_handler(
    resume_id: i32,
    maybe_auth: MaybeAuthSession,
) -> Result<Json<Response<Resume>>, Custom<Json<Response<String>>>> {
    let user_id_value = maybe_auth.0.map(|a| a.user_id);
    let resume = read::list_resume(resume_id, user_id_value).map_err(|err| match err {
        ApplicationError::NotFound(msg) => {
            Custom(rocket::http::Status::NotFound, Json(Response { body: msg }))
        }
        ApplicationError::Forbidden => Custom(
            rocket::http::Status::Forbidden,
            Json(Response {
                body: "Forbidden".to_string(),
            }),
        ),
        ApplicationError::Conflict(msg) => {
            Custom(rocket::http::Status::Conflict, Json(Response { body: msg }))
        }
        ApplicationError::BadRequest(msg) => Custom(
            rocket::http::Status::BadRequest,
            Json(Response { body: msg }),
        ),
        ApplicationError::Internal(msg) => Custom(
            rocket::http::Status::InternalServerError,
            Json(Response { body: msg }),
        ),
        ApplicationError::Unauthorized => Custom(
            rocket::http::Status::Unauthorized,
            Json(Response {
                body: "Unauthorized".to_string(),
            }),
        ),
    })?;

    Ok(Json(Response { body: resume }))
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
) -> Result<Custom<Json<Response<Resume>>>, Conflict<Json<Response<String>>>> {
    match create::create_resume(auth.user_id, resume.into_inner()) {
        Ok(resume) => {
            hub.publish_resume_changed(resume.id, ResumeChangedAction::Created);
            Ok(Custom(
                rocket::http::Status::Created,
                Json(Response { body: resume }),
            ))
        }
        Err(ApplicationError::Conflict(msg)) => Err(Conflict(Json(Response { body: msg }))),
        Err(err) => Err(Conflict(Json(Response {
            body: format!("{:?}", err),
        }))),
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
) -> Result<Json<Response<Resume>>, Custom<Json<Response<String>>>> {
    match update::update_resume(auth.user_id, resume_id, resume.into_inner()) {
        Ok(updated) => {
            hub.publish_resume_changed(
                updated.id,
                ResumeChangedAction::Updated(crate::realtime::SectionType::PersonalInfo),
            );
            Ok(Json(Response { body: updated }))
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
        Err(ApplicationError::Unauthorized) => Err(Custom(
            rocket::http::Status::Unauthorized,
            Json(Response {
                body: "Unauthorized".to_string(),
            }),
        )),
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
) -> Result<NoContent, Custom<Json<Response<String>>>> {
    match delete::delete_resume(auth.user_id, resume_id) {
        Ok(()) => {
            hub.publish_resume_changed(resume_id, ResumeChangedAction::Deleted);
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
        Err(ApplicationError::Unauthorized) => Err(Custom(
            rocket::http::Status::Unauthorized,
            Json(Response {
                body: "Unauthorized".to_string(),
            }),
        )),
    }
}
