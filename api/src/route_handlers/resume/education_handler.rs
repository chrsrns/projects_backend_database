use application::resume::education;
use domain::models::{
    Education, EducationKeyPoint, NewEducationKeyPointRequest, NewEducationRequest,
    UpdateEducationKeyPoint, UpdateEducationRequest,
};
use rocket::State;
use rocket::response::status::{Custom, NoContent};
use rocket::serde::json::Json;
use rocket::{delete as rocket_delete, get, post, put};
use shared::response_models::Response;
use uuid::Uuid;

use super::{CustomJsonResult, JsonResult, NoContentResult};
use crate::auth::{AuthSession, MaybeAuthSession};
use crate::realtime::Hub;

#[utoipa::path(
    get,
    path = "/resume/{resume_id}/education",
    tag = "Education",
    params(
        ("resume_id" = Uuid, Path, description = "Resume id")
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
    resume_id: Uuid,
    maybe_auth: MaybeAuthSession,
) -> JsonResult<Vec<Education>> {
    let user_id_value = maybe_auth.0.map(|a| a.user_id);
    let items = education::list_educations(resume_id, user_id_value)?;
    Ok(Json(Response { body: items }))
}

#[utoipa::path(
    post,
    path = "/resume/{resume_id}/education",
    tag = "Education",
    security(("bearerAuth" = [])),
    params(
        ("resume_id" = Uuid, Path, description = "Resume id")
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
    resume_id: Uuid,
    payload: Json<NewEducationRequest>,
) -> CustomJsonResult<Education> {
    let item =
        education::create_education(auth.user_id, resume_id, payload.into_inner(), hub.inner())?;
    Ok(Custom(
        rocket::http::Status::Created,
        Json(Response { body: item }),
    ))
}

#[utoipa::path(
    put,
    path = "/education/{education_id}",
    tag = "Education",
    security(("bearerAuth" = [])),
    params(
        ("education_id" = i32, Path, description = "Education id")
    ),
    request_body(content = UpdateEducationRequest, content_type = "application/json"),
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
    payload: Json<UpdateEducationRequest>,
) -> JsonResult<Education> {
    let item = education::update_education(
        auth.user_id,
        education_id,
        payload.into_inner(),
        hub.inner(),
    )?;
    Ok(Json(Response { body: item }))
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
) -> NoContentResult {
    education::delete_education(auth.user_id, education_id, hub.inner())?;
    Ok(NoContent)
}

#[utoipa::path(
    get,
    path = "/resume/{resume_id}/education/{education_id}/key_points",
    tag = "Education",
    params(
        ("resume_id" = Uuid, Path, description = "Resume id"),
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
    resume_id: Uuid,
    education_id: i32,
    maybe_auth: MaybeAuthSession,
) -> JsonResult<Vec<EducationKeyPoint>> {
    let user_id_value = maybe_auth.0.map(|a| a.user_id);
    let items = education::list_education_key_points(resume_id, education_id, user_id_value)?;
    Ok(Json(Response { body: items }))
}

#[utoipa::path(
    post,
    path = "/resume/{resume_id}/education/{education_id}/key_points",
    tag = "Education",
    security(("bearerAuth" = [])),
    params(
        ("resume_id" = Uuid, Path, description = "Resume id"),
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
    resume_id: Uuid,
    education_id: i32,
    payload: Json<NewEducationKeyPointRequest>,
) -> CustomJsonResult<EducationKeyPoint> {
    let item = education::create_education_key_point(
        auth.user_id,
        resume_id,
        education_id,
        payload.into_inner(),
        hub.inner(),
    )?;
    Ok(Custom(
        rocket::http::Status::Created,
        Json(Response { body: item }),
    ))
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
) -> JsonResult<EducationKeyPoint> {
    let item = education::update_education_key_point(
        auth.user_id,
        key_point_id,
        payload.into_inner(),
        hub.inner(),
    )?;
    Ok(Json(Response { body: item }))
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
) -> NoContentResult {
    education::delete_education_key_point(auth.user_id, key_point_id, hub.inner())?;
    Ok(NoContent)
}
