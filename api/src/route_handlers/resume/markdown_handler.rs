use std::io::Cursor;

use application::resume::{markdown_export, markdown_import};
use domain::models::ResumeView;
use rocket::data::{self, FromData, ToByteUnit};
use rocket::http::Status;
use rocket::request::Request;
use rocket::response::status::Custom;
use rocket::response::{Responder, Response as RocketResponse};
use rocket::serde::json::Json;
use rocket::{State, get, post};
use shared::markdown::{MarkdownError, markdown_to_resume};
use shared::response_models::Response;
use shared::resume_document::{
    MarkdownValidationError, MarkdownValidationReport, RESUME_DOCUMENT_GENERATOR,
    RESUME_DOCUMENT_SCHEMA_VERSION, ResumeDocument, ResumeDocumentEnvelope,
};

use super::{CustomJsonResult, JsonResult};
use crate::auth::{AuthSession, MaybeAuthSession};
use crate::error::ApiError;
use crate::realtime::Hub;

const MAX_MARKDOWN_SIZE: u64 = 1_048_576; // 1 MiB

pub struct LimitedMarkdown(pub String);

#[rocket::async_trait]
impl<'r> FromData<'r> for LimitedMarkdown {
    type Error = String;

    async fn from_data(_req: &'r Request<'_>, data: data::Data<'r>) -> data::Outcome<'r, Self> {
        let limit = (MAX_MARKDOWN_SIZE + 1).bytes();
        let string = match data.open(limit).into_string().await {
            Ok(string) if string.len() <= MAX_MARKDOWN_SIZE as usize => string.into_inner(),
            Ok(_) => {
                return data::Outcome::Error((
                    Status::PayloadTooLarge,
                    "Markdown exceeds maximum size of 1 MiB".into(),
                ));
            }
            Err(e) => return data::Outcome::Error((Status::InternalServerError, e.to_string())),
        };
        data::Outcome::Success(LimitedMarkdown(string))
    }
}

pub struct MarkdownResponse(String);

type MarkdownResult = Result<MarkdownResponse, ApiError>;

impl<'r> Responder<'r, 'static> for MarkdownResponse {
    fn respond_to(self, _req: &'r Request<'_>) -> rocket::response::Result<'static> {
        let mut builder = RocketResponse::build();
        builder.status(Status::Ok);
        builder.raw_header("Content-Type", "text/markdown; charset=utf-8");
        builder.sized_body(self.0.len(), Cursor::new(self.0)).ok()
    }
}

#[utoipa::path(
    get,
    path = "/resume/{resume_id}/export/markdown",
    tag = "Resumes",
    params(
        ("resume_id" = i32, Path, description = "Resume ID")
    ),
    responses(
        (status = 200, description = "Markdown representation", body = String, content_type = "text/markdown"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
    )
)]
#[get("/resume/<resume_id>/export/markdown")]
pub fn export_resume_markdown(resume_id: i32, maybe_auth: MaybeAuthSession) -> MarkdownResult {
    let user_id_value = maybe_auth.0.map(|a| a.user_id);
    let markdown = markdown_export::export_resume_markdown(resume_id, user_id_value)?;
    Ok(MarkdownResponse(markdown))
}

#[utoipa::path(
    post,
    path = "/resume/import/markdown",
    tag = "Resumes",
    security(("bearerAuth" = [])),
    request_body = String,
    responses(
        (status = 200, description = "Updated resume", body = Response<ResumeView>, content_type = "application/json"),
        (status = 201, description = "Created resume", body = Response<ResumeView>, content_type = "application/json"),
        (status = 400, description = "Invalid Markdown"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 413, description = "Markdown payload exceeds 1 MiB limit"),
    )
)]
#[post(
    "/resume/import/markdown",
    format = "text/markdown",
    data = "<markdown>"
)]
pub fn import_resume_markdown(
    auth: AuthSession,
    markdown: LimitedMarkdown,
    hub: &State<Hub>,
) -> CustomJsonResult<ResumeView> {
    let (resume, created) =
        markdown_import::import_resume_markdown(&markdown.0, auth.user_id, None, hub.inner())?;
    if created {
        Ok(Custom(Status::Created, Json(Response { body: resume })))
    } else {
        Ok(Custom(Status::Ok, Json(Response { body: resume })))
    }
}

#[utoipa::path(
    post,
    path = "/resume/{resume_id}/import/markdown",
    tag = "Resumes",
    security(("bearerAuth" = [])),
    params(
        ("resume_id" = i32, Path, description = "Resume ID to import into")
    ),
    request_body = String,
    responses(
        (status = 200, description = "Updated resume", body = Response<ResumeView>, content_type = "application/json"),
        (status = 400, description = "Invalid Markdown or front-matter marker mismatch"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
        (status = 413, description = "Markdown payload exceeds 1 MiB limit"),
    )
)]
#[post(
    "/resume/<resume_id>/import/markdown",
    format = "text/markdown",
    data = "<markdown>"
)]
pub fn import_resume_markdown_into(
    auth: AuthSession,
    markdown: LimitedMarkdown,
    hub: &State<Hub>,
    resume_id: i32,
) -> CustomJsonResult<ResumeView> {
    let (resume, _) = markdown_import::import_resume_markdown(
        &markdown.0,
        auth.user_id,
        Some(resume_id),
        hub.inner(),
    )?;
    Ok(Custom(Status::Ok, Json(Response { body: resume })))
}

#[utoipa::path(
    post,
    path = "/resume/validate/markdown",
    tag = "Resumes",
    request_body(content = String, content_type = "text/markdown"),
    responses(
        (status = 200, description = "Markdown validation report", body = Response<MarkdownValidationReport>, content_type = "application/json"),
        (status = 413, description = "Markdown payload exceeds 1 MiB limit"),
        (status = 500, description = "Internal server error"),
    )
)]
#[post(
    "/resume/validate/markdown",
    format = "text/markdown",
    data = "<markdown>"
)]
pub fn validate_resume_markdown(
    markdown: LimitedMarkdown,
) -> Json<Response<MarkdownValidationReport>> {
    let report = match markdown_to_resume(&markdown.0) {
        Ok(_) => MarkdownValidationReport {
            valid: true,
            errors: Vec::new(),
        },
        Err(MarkdownError::InvalidMarkdown(message)) => MarkdownValidationReport {
            valid: false,
            errors: vec![MarkdownValidationError {
                section: None,
                message,
            }],
        },
    };
    Json(Response { body: report })
}

#[utoipa::path(
    post,
    path = "/resume/convert/markdown",
    tag = "Resumes",
    request_body(content = String, content_type = "text/markdown"),
    responses(
        (status = 200, description = "Resume document envelope", body = Response<ResumeDocumentEnvelope>, content_type = "application/json"),
        (status = 400, description = "Invalid Markdown"),
        (status = 413, description = "Markdown payload exceeds 1 MiB limit"),
        (status = 500, description = "Internal server error"),
    )
)]
#[post(
    "/resume/convert/markdown",
    format = "text/markdown",
    data = "<markdown>"
)]
pub fn convert_resume_markdown(markdown: LimitedMarkdown) -> JsonResult<ResumeDocumentEnvelope> {
    match markdown_to_resume(&markdown.0) {
        Ok(parsed) => {
            let document: ResumeDocument = parsed.into();
            Ok(Json(Response {
                body: ResumeDocumentEnvelope {
                    schema_version: RESUME_DOCUMENT_SCHEMA_VERSION,
                    generator: RESUME_DOCUMENT_GENERATOR.to_string(),
                    document,
                },
            }))
        }
        Err(MarkdownError::InvalidMarkdown(message)) => Err(ApiError(Custom(
            Status::BadRequest,
            Json(Response { body: message }),
        ))),
    }
}

const MARKDOWN_FORMAT: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../MARKDOWN_FORMAT.md"
));

#[utoipa::path(
    get,
    path = "/resume/markdown-format",
    tag = "Resumes",
    responses(
        (status = 200, description = "Markdown format specification", body = String, content_type = "text/markdown"),
    )
)]
#[get("/resume/markdown-format")]
pub fn get_markdown_format() -> MarkdownResult {
    Ok(MarkdownResponse(MARKDOWN_FORMAT.to_string()))
}

#[utoipa::path(
    get,
    path = "/resume/document-schema",
    tag = "Resumes",
    responses(
        (status = 200, description = "OpenAPI 3.0 schema object for ResumeDocument", content_type = "application/json"),
    )
)]
#[get("/resume/document-schema")]
pub fn resume_document_schema() -> Json<serde_json::Value> {
    Json(crate::openapi::resume_document_schema_json())
}
