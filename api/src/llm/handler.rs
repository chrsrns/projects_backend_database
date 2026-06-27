use std::sync::Arc;

use application::llm::{
    GenerateContentRequest, GenerateContentResponse, LlmClient, generate_content,
};
use rocket::http::Status;
use rocket::response::status::Custom;
use rocket::serde::json::{Error as JsonError, Json, Value};
use rocket::{State, post};
use shared::response_models::Response;

use super::CustomJsonResult;
use crate::error::map_application_error;

#[utoipa::path(
    post,
    path = "/llm/gemini",
    tag = "LLM",
    request_body(content = GenerateContentRequest, content_type = "application/json"),
    responses(
        (status = 200, description = "OK", body = Response<GenerateContentResponse>, content_type = "application/json"),
        (status = 400, description = "Bad Request", body = Response<String>, content_type = "application/json"),
        (status = 500, description = "Internal Server Error", body = Response<String>, content_type = "application/json")
    )
)]
#[post("/llm/gemini", format = "application/json", data = "<payload>")]
pub async fn generate_handler(
    payload: Result<Json<Value>, JsonError<'_>>,
    client: &State<Arc<dyn LlmClient + Send + Sync>>,
) -> CustomJsonResult<GenerateContentResponse> {
    let payload = match payload {
        Ok(payload) => payload,
        Err(err) => {
            return Err(Custom(
                Status::BadRequest,
                Json(Response {
                    body: format!("Invalid request body: {err}"),
                }),
            ));
        }
    };

    let request: GenerateContentRequest = match serde_json::from_value(payload.into_inner()) {
        Ok(request) => request,
        Err(err) => {
            return Err(Custom(
                Status::BadRequest,
                Json(Response {
                    body: format!("Invalid request body: {err}"),
                }),
            ));
        }
    };

    let client_ref: &dyn LlmClient = client.inner().as_ref();
    match generate_content(client_ref, request).await {
        Ok(response) => Ok(Custom(Status::Ok, Json(Response { body: response }))),
        Err(err) => Err(map_application_error(err)),
    }
}
