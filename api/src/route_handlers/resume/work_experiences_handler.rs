use application::error::ApplicationError;
use application::resume::work_experiences;
use domain::models::{
    NewWorkExperienceKeyPointRequest, NewWorkExperienceRequest, UpdateWorkExperience,
    UpdateWorkExperienceKeyPoint, WorkExperience, WorkExperienceKeyPoint,
};
use rocket::State;
use rocket::response::status::{Custom, NoContent};
use rocket::serde::json::Json;
use rocket::{delete as rocket_delete, get, post, put};
use shared::response_models::Response;

use crate::auth::{AuthSession, MaybeAuthSession};
use crate::realtime::{Hub, ResumeChangedAction};

#[utoipa::path(
    get,
    path = "/resume/{resume_id}/work_experiences",
    tag = "WorkExperiences",
    params(
        ("resume_id" = i32, Path, description = "Resume id")
    ),
    responses(
        (status = 200, description = "OK", body = Response<Vec<WorkExperience>>, content_type = "application/json"),
        (status = 401, description = "Unauthorized", body = Response<String>, content_type = "application/json"),
        (status = 403, description = "Forbidden", body = Response<String>, content_type = "application/json"),
        (status = 404, description = "Not Found", body = Response<String>, content_type = "application/json")
    )
)]
#[get("/resume/<resume_id>/work_experiences")]
pub fn list_work_experiences_handler(
    resume_id: i32,
    maybe_auth: MaybeAuthSession,
) -> Result<Json<Response<Vec<WorkExperience>>>, Custom<Json<Response<String>>>> {
    let user_id_value = maybe_auth.0.map(|a| a.user_id);

    match work_experiences::list_work_experiences(resume_id, user_id_value) {
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
    path = "/resume/{resume_id}/work_experiences",
    tag = "WorkExperiences",
    security(("bearerAuth" = [])),
    params(
        ("resume_id" = i32, Path, description = "Resume id")
    ),
    request_body(content = NewWorkExperienceRequest, content_type = "application/json"),
    responses(
        (status = 201, description = "Created", body = Response<WorkExperience>, content_type = "application/json"),
        (status = 401, description = "Unauthorized", body = Response<String>, content_type = "application/json"),
        (status = 403, description = "Forbidden", body = Response<String>, content_type = "application/json"),
        (status = 404, description = "Not Found", body = Response<String>, content_type = "application/json")
    )
)]
#[post(
    "/resume/<resume_id>/work_experiences",
    format = "application/json",
    data = "<payload>"
)]
pub fn create_work_experience_handler(
    auth: AuthSession,
    hub: &State<Hub>,
    resume_id: i32,
    payload: Json<NewWorkExperienceRequest>,
) -> Result<Custom<Json<Response<WorkExperience>>>, Custom<Json<Response<String>>>> {
    match work_experiences::create_work_experience(auth.user_id, resume_id, payload.into_inner()) {
        Ok(item) => {
            hub.publish_resume_changed(resume_id, ResumeChangedAction::Updated);
            Ok(Custom(
                rocket::http::Status::Created,
                Json(Response { body: item }),
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
    path = "/work_experiences/{work_id}",
    tag = "WorkExperiences",
    security(("bearerAuth" = [])),
    params(
        ("work_id" = i32, Path, description = "Work experience id")
    ),
    request_body(content = UpdateWorkExperience, content_type = "application/json"),
    responses(
        (status = 200, description = "OK", body = Response<WorkExperience>, content_type = "application/json"),
        (status = 401, description = "Unauthorized", body = Response<String>, content_type = "application/json"),
        (status = 403, description = "Forbidden", body = Response<String>, content_type = "application/json"),
        (status = 404, description = "Not Found", body = Response<String>, content_type = "application/json")
    )
)]
#[put(
    "/work_experiences/<work_id>",
    format = "application/json",
    data = "<payload>"
)]
pub fn update_work_experience_handler(
    auth: AuthSession,
    hub: &State<Hub>,
    work_id: i32,
    payload: Json<UpdateWorkExperience>,
) -> Result<Json<Response<WorkExperience>>, Custom<Json<Response<String>>>> {
    match work_experiences::update_work_experience(auth.user_id, work_id, payload.into_inner()) {
        Ok(item) => {
            hub.publish_resume_changed(item.resume_id, ResumeChangedAction::Updated);
            Ok(Json(Response { body: item }))
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
    path = "/work_experiences/{work_id}",
    tag = "WorkExperiences",
    security(("bearerAuth" = [])),
    params(
        ("work_id" = i32, Path, description = "Work experience id")
    ),
    responses(
        (status = 204, description = "No Content"),
        (status = 401, description = "Unauthorized", body = Response<String>, content_type = "application/json"),
        (status = 403, description = "Forbidden", body = Response<String>, content_type = "application/json"),
        (status = 404, description = "Not Found", body = Response<String>, content_type = "application/json")
    )
)]
#[rocket_delete("/work_experiences/<work_id>")]
pub fn delete_work_experience_handler(
    auth: AuthSession,
    hub: &State<Hub>,
    work_id: i32,
) -> Result<NoContent, Custom<Json<Response<String>>>> {
    match work_experiences::delete_work_experience(auth.user_id, work_id) {
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

#[utoipa::path(
    get,
    path = "/resume/{resume_id}/work_experiences/{work_id}/key_points",
    tag = "WorkExperienceKeyPoints",
    params(
        ("resume_id" = i32, Path, description = "Resume id"),
        ("work_id" = i32, Path, description = "Work experience id")
    ),
    responses(
        (status = 200, description = "OK", body = Response<Vec<WorkExperienceKeyPoint>>, content_type = "application/json"),
        (status = 401, description = "Unauthorized", body = Response<String>, content_type = "application/json"),
        (status = 403, description = "Forbidden", body = Response<String>, content_type = "application/json"),
        (status = 404, description = "Not Found", body = Response<String>, content_type = "application/json")
    )
)]
#[get("/resume/<resume_id>/work_experiences/<work_id>/key_points")]
pub fn list_work_experience_key_points_handler(
    resume_id: i32,
    work_id: i32,
    maybe_auth: MaybeAuthSession,
) -> Result<Json<Response<Vec<WorkExperienceKeyPoint>>>, Custom<Json<Response<String>>>> {
    let user_id_value = maybe_auth.0.map(|a| a.user_id);

    match work_experiences::list_work_experience_key_points(resume_id, work_id, user_id_value) {
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
    path = "/resume/{resume_id}/work_experiences/{work_id}/key_points",
    tag = "WorkExperienceKeyPoints",
    security(("bearerAuth" = [])),
    params(
        ("resume_id" = i32, Path, description = "Resume id"),
        ("work_id" = i32, Path, description = "Work experience id")
    ),
    request_body(content = NewWorkExperienceKeyPointRequest, content_type = "application/json"),
    responses(
        (status = 201, description = "Created", body = Response<WorkExperienceKeyPoint>, content_type = "application/json"),
        (status = 401, description = "Unauthorized", body = Response<String>, content_type = "application/json"),
        (status = 403, description = "Forbidden", body = Response<String>, content_type = "application/json"),
        (status = 404, description = "Not Found", body = Response<String>, content_type = "application/json")
    )
)]
#[post(
    "/resume/<resume_id>/work_experiences/<work_id>/key_points",
    format = "application/json",
    data = "<payload>"
)]
pub fn create_work_experience_key_point_handler(
    auth: AuthSession,
    hub: &State<Hub>,
    resume_id: i32,
    work_id: i32,
    payload: Json<NewWorkExperienceKeyPointRequest>,
) -> Result<Custom<Json<Response<WorkExperienceKeyPoint>>>, Custom<Json<Response<String>>>> {
    match work_experiences::create_work_experience_key_point(
        auth.user_id,
        resume_id,
        work_id,
        payload.into_inner(),
    ) {
        Ok(item) => {
            hub.publish_resume_changed(resume_id, ResumeChangedAction::Updated);
            Ok(Custom(
                rocket::http::Status::Created,
                Json(Response { body: item }),
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
    path = "/work_experience_key_points/{key_point_id}",
    tag = "WorkExperienceKeyPoints",
    security(("bearerAuth" = [])),
    params(
        ("key_point_id" = i32, Path, description = "Key point id")
    ),
    request_body(content = UpdateWorkExperienceKeyPoint, content_type = "application/json"),
    responses(
        (status = 200, description = "OK", body = Response<WorkExperienceKeyPoint>, content_type = "application/json"),
        (status = 401, description = "Unauthorized", body = Response<String>, content_type = "application/json"),
        (status = 403, description = "Forbidden", body = Response<String>, content_type = "application/json"),
        (status = 404, description = "Not Found", body = Response<String>, content_type = "application/json")
    )
)]
#[put(
    "/work_experience_key_points/<key_point_id>",
    format = "application/json",
    data = "<payload>"
)]
pub fn update_work_experience_key_point_handler(
    auth: AuthSession,
    hub: &State<Hub>,
    key_point_id: i32,
    payload: Json<UpdateWorkExperienceKeyPoint>,
) -> Result<Json<Response<WorkExperienceKeyPoint>>, Custom<Json<Response<String>>>> {
    match work_experiences::update_work_experience_key_point(
        auth.user_id,
        key_point_id,
        payload.into_inner(),
    ) {
        Ok((item, resume_id)) => {
            hub.publish_resume_changed(resume_id, ResumeChangedAction::Updated);
            Ok(Json(Response { body: item }))
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
    path = "/work_experience_key_points/{key_point_id}",
    tag = "WorkExperienceKeyPoints",
    security(("bearerAuth" = [])),
    params(
        ("key_point_id" = i32, Path, description = "Key point id")
    ),
    responses(
        (status = 204, description = "No Content"),
        (status = 401, description = "Unauthorized", body = Response<String>, content_type = "application/json"),
        (status = 403, description = "Forbidden", body = Response<String>, content_type = "application/json"),
        (status = 404, description = "Not Found", body = Response<String>, content_type = "application/json")
    )
)]
#[rocket_delete("/work_experience_key_points/<key_point_id>")]
pub fn delete_work_experience_key_point_handler(
    auth: AuthSession,
    hub: &State<Hub>,
    key_point_id: i32,
) -> Result<NoContent, Custom<Json<Response<String>>>> {
    match work_experiences::delete_work_experience_key_point(auth.user_id, key_point_id) {
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
