use application::error::ApplicationError;
use application::resume::education;
use domain::models::{
    Education, EducationKeyPoint, NewEducationKeyPointRequest, NewEducationRequest,
    UpdateEducation, UpdateEducationKeyPoint,
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
    path = "/resume/{resume_id}/education",
    tag = "Education",
    params(
        ("resume_id" = i32, Path, description = "Resume id")
    ),
    responses(
        (status = 200, description = "OK", body = Response<Vec<Education>>, content_type = "application/json"),
        (status = 401, description = "Unauthorized", body = Response<String>, content_type = "application/json"),
        (status = 403, description = "Forbidden", body = Response<String>, content_type = "application/json"),
        (status = 404, description = "Not Found", body = Response<String>, content_type = "application/json")
    )
)]
#[get("/resume/<resume_id>/education")]
pub fn list_educations_handler(
    resume_id: i32,
    maybe_auth: MaybeAuthSession,
) -> Result<Json<Response<Vec<Education>>>, Custom<Json<Response<String>>>> {
    let user_id_value = maybe_auth.0.map(|a| a.user_id);

    match education::list_educations(resume_id, user_id_value) {
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
    path = "/resume/{resume_id}/education",
    tag = "Education",
    security(("bearerAuth" = [])),
    params(
        ("resume_id" = i32, Path, description = "Resume id")
    ),
    request_body(content = NewEducationRequest, content_type = "application/json"),
    responses(
        (status = 201, description = "Created", body = Response<Education>, content_type = "application/json"),
        (status = 401, description = "Unauthorized", body = Response<String>, content_type = "application/json"),
        (status = 403, description = "Forbidden", body = Response<String>, content_type = "application/json"),
        (status = 404, description = "Not Found", body = Response<String>, content_type = "application/json")
    )
)]
#[post(
    "/resume/<resume_id>/education",
    format = "application/json",
    data = "<payload>"
)]
pub fn create_education_handler(
    auth: AuthSession,
    hub: &State<Hub>,
    resume_id: i32,
    payload: Json<NewEducationRequest>,
) -> Result<Custom<Json<Response<Education>>>, Custom<Json<Response<String>>>> {
    match education::create_education(auth.user_id, resume_id, payload.into_inner()) {
        Ok(item) => {
            hub.publish_resume_changed(
                resume_id,
                ResumeChangedAction::Updated(crate::realtime::SectionType::Education),
            );
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
    path = "/education/{education_id}",
    tag = "Education",
    security(("bearerAuth" = [])),
    params(
        ("education_id" = i32, Path, description = "Education id")
    ),
    request_body(content = UpdateEducation, content_type = "application/json"),
    responses(
        (status = 200, description = "OK", body = Response<Education>, content_type = "application/json"),
        (status = 401, description = "Unauthorized", body = Response<String>, content_type = "application/json"),
        (status = 403, description = "Forbidden", body = Response<String>, content_type = "application/json"),
        (status = 404, description = "Not Found", body = Response<String>, content_type = "application/json")
    )
)]
#[put(
    "/education/<education_id>",
    format = "application/json",
    data = "<payload>"
)]
pub fn update_education_handler(
    auth: AuthSession,
    hub: &State<Hub>,
    education_id: i32,
    payload: Json<UpdateEducation>,
) -> Result<Json<Response<Education>>, Custom<Json<Response<String>>>> {
    match education::update_education(auth.user_id, education_id, payload.into_inner()) {
        Ok(item) => {
            hub.publish_resume_changed(
                item.resume_id,
                ResumeChangedAction::Updated(crate::realtime::SectionType::Education),
            );
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
    path = "/education/{education_id}",
    tag = "Education",
    security(("bearerAuth" = [])),
    params(
        ("education_id" = i32, Path, description = "Education id")
    ),
    responses(
        (status = 204, description = "No Content"),
        (status = 401, description = "Unauthorized", body = Response<String>, content_type = "application/json"),
        (status = 403, description = "Forbidden", body = Response<String>, content_type = "application/json"),
        (status = 404, description = "Not Found", body = Response<String>, content_type = "application/json")
    )
)]
#[rocket_delete("/education/<education_id>")]
pub fn delete_education_handler(
    auth: AuthSession,
    hub: &State<Hub>,
    education_id: i32,
) -> Result<NoContent, Custom<Json<Response<String>>>> {
    match education::delete_education(auth.user_id, education_id) {
        Ok(resume_id) => {
            hub.publish_resume_changed(
                resume_id,
                ResumeChangedAction::Updated(crate::realtime::SectionType::Education),
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

#[utoipa::path(
    get,
    path = "/resume/{resume_id}/education/{education_id}/key_points",
    tag = "Education",
    params(
        ("resume_id" = i32, Path, description = "Resume id"),
        ("education_id" = i32, Path, description = "Education id")
    ),
    responses(
        (status = 200, description = "OK", body = Response<Vec<EducationKeyPoint>>, content_type = "application/json"),
        (status = 401, description = "Unauthorized", body = Response<String>, content_type = "application/json"),
        (status = 403, description = "Forbidden", body = Response<String>, content_type = "application/json"),
        (status = 404, description = "Not Found", body = Response<String>, content_type = "application/json")
    )
)]
#[get("/resume/<resume_id>/education/<education_id>/key_points")]
pub fn list_education_key_points_handler(
    resume_id: i32,
    education_id: i32,
    maybe_auth: MaybeAuthSession,
) -> Result<Json<Response<Vec<EducationKeyPoint>>>, Custom<Json<Response<String>>>> {
    let user_id_value = maybe_auth.0.map(|a| a.user_id);

    match education::list_education_key_points(resume_id, education_id, user_id_value) {
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
    path = "/resume/{resume_id}/education/{education_id}/key_points",
    tag = "Education",
    security(("bearerAuth" = [])),
    params(
        ("resume_id" = i32, Path, description = "Resume id"),
        ("education_id" = i32, Path, description = "Education id")
    ),
    request_body(content = NewEducationKeyPointRequest, content_type = "application/json"),
    responses(
        (status = 201, description = "Created", body = Response<EducationKeyPoint>, content_type = "application/json"),
        (status = 401, description = "Unauthorized", body = Response<String>, content_type = "application/json"),
        (status = 403, description = "Forbidden", body = Response<String>, content_type = "application/json"),
        (status = 404, description = "Not Found", body = Response<String>, content_type = "application/json")
    )
)]
#[post(
    "/resume/<resume_id>/education/<education_id>/key_points",
    format = "application/json",
    data = "<payload>"
)]
pub fn create_education_key_point_handler(
    auth: AuthSession,
    hub: &State<Hub>,
    resume_id: i32,
    education_id: i32,
    payload: Json<NewEducationKeyPointRequest>,
) -> Result<Custom<Json<Response<EducationKeyPoint>>>, Custom<Json<Response<String>>>> {
    match education::create_education_key_point(
        auth.user_id,
        resume_id,
        education_id,
        payload.into_inner(),
    ) {
        Ok(item) => {
            hub.publish_resume_changed(
                resume_id,
                ResumeChangedAction::Updated(crate::realtime::SectionType::Education),
            );
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
    path = "/education_key_points/{key_point_id}",
    tag = "Education",
    security(("bearerAuth" = [])),
    params(
        ("key_point_id" = i32, Path, description = "Key point id")
    ),
    request_body(content = UpdateEducationKeyPoint, content_type = "application/json"),
    responses(
        (status = 200, description = "OK", body = Response<EducationKeyPoint>, content_type = "application/json"),
        (status = 401, description = "Unauthorized", body = Response<String>, content_type = "application/json"),
        (status = 403, description = "Forbidden", body = Response<String>, content_type = "application/json"),
        (status = 404, description = "Not Found", body = Response<String>, content_type = "application/json")
    )
)]
#[put(
    "/education_key_points/<key_point_id>",
    format = "application/json",
    data = "<payload>"
)]
pub fn update_education_key_point_handler(
    auth: AuthSession,
    hub: &State<Hub>,
    key_point_id: i32,
    payload: Json<UpdateEducationKeyPoint>,
) -> Result<Json<Response<EducationKeyPoint>>, Custom<Json<Response<String>>>> {
    match education::update_education_key_point(auth.user_id, key_point_id, payload.into_inner()) {
        Ok((item, resume_id)) => {
            hub.publish_resume_changed(
                resume_id,
                ResumeChangedAction::Updated(crate::realtime::SectionType::Education),
            );
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
    path = "/education_key_points/{key_point_id}",
    tag = "Education",
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
#[rocket_delete("/education_key_points/<key_point_id>")]
pub fn delete_education_key_point_handler(
    auth: AuthSession,
    hub: &State<Hub>,
    key_point_id: i32,
) -> Result<NoContent, Custom<Json<Response<String>>>> {
    match education::delete_education_key_point(auth.user_id, key_point_id) {
        Ok(resume_id) => {
            hub.publish_resume_changed(
                resume_id,
                ResumeChangedAction::Updated(crate::realtime::SectionType::Education),
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
