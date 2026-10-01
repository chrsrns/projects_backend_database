use application::resume::variant;
use domain::models::{NewVariantRequest, ResumeView};
use rocket::State;
use rocket::http::Status;
use rocket::response::status::Custom;
use rocket::serde::json::Json;
use rocket::{get, post};
use shared::response_models::Response;

use super::{CustomJsonResult, JsonResult};
use crate::auth::{AuthSession, MaybeAuthSession};
use crate::error::map_application_error;
use crate::realtime::{Hub, ResumeChangedAction};

#[utoipa::path(
    post,
    path = "/resume/{resume_id}/variants",
    tag = "Resumes",
    security(("bearerAuth" = [])),
    params(
        ("resume_id" = i32, Path, description = "Base resume id")
    ),
    request_body(content = NewVariantRequest, content_type = "application/json"),
    responses(
        (status = 201, description = "Created", body = Response<ResumeView>, content_type = "application/json"),
        (status = 400, description = "Bad Request", body = Response<String>, content_type = "application/json"),
        (status = 401, description = "Unauthorized", body = Response<String>, content_type = "application/json"),
        (status = 403, description = "Forbidden", body = Response<String>, content_type = "application/json"),
        (status = 404, description = "Not Found", body = Response<String>, content_type = "application/json")
    )
)]
#[post(
    "/resume/<resume_id>/variants",
    format = "application/json",
    data = "<request>"
)]
pub fn create_variant_handler(
    auth: AuthSession,
    hub: &State<Hub>,
    resume_id: i32,
    request: Json<NewVariantRequest>,
) -> CustomJsonResult<ResumeView> {
    match variant::create_variant(auth.user_id, resume_id, request.into_inner()) {
        Ok(created) => {
            hub.publish_resume_changed(created.id, ResumeChangedAction::Created);
            Ok(Custom(Status::Created, Json(Response { body: created })))
        }
        Err(err) => Err(map_application_error(err)),
    }
}

#[utoipa::path(
    get,
    path = "/resume/{resume_id}/variants",
    tag = "Resumes",
    params(
        ("resume_id" = i32, Path, description = "Base resume id")
    ),
    responses(
        (status = 200, description = "OK", body = Response<Vec<ResumeView>>, content_type = "application/json"),
        (status = 404, description = "Not Found", body = Response<String>, content_type = "application/json")
    )
)]
#[get("/resume/<resume_id>/variants")]
pub fn list_variants_handler(
    resume_id: i32,
    maybe_auth: MaybeAuthSession,
) -> JsonResult<Vec<ResumeView>> {
    let user_id_value = maybe_auth.0.map(|auth| auth.user_id);
    match variant::list_variants(resume_id, user_id_value) {
        Ok(variants) => Ok(Json(Response { body: variants })),
        Err(err) => Err(map_application_error(err)),
    }
}
