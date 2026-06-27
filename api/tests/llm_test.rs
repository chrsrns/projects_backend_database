use std::sync::Arc;

use application::error::ApplicationError;
use application::llm::{GenerateContentRequest, GenerateContentResponse, LlmClient, UsageMetadata};
use rocket::http::{ContentType, Status};
use rocket::local::blocking::Client;
use serde_json::json;
use shared::node_config::NodeConfig;

#[derive(Default)]
struct MockLlmClient {
    response: Option<GenerateContentResponse>,
    error: Option<ApplicationError>,
}

impl MockLlmClient {
    fn new(response: GenerateContentResponse) -> Self {
        Self {
            response: Some(response),
            error: None,
        }
    }

    fn with_error(error: ApplicationError) -> Self {
        Self {
            response: None,
            error: Some(error),
        }
    }
}

#[rocket::async_trait]
impl LlmClient for MockLlmClient {
    async fn generate_content(
        &self,
        _request: GenerateContentRequest,
    ) -> Result<GenerateContentResponse, ApplicationError> {
        match (&self.response, &self.error) {
            (Some(response), _) => Ok(response.clone()),
            (_, Some(error)) => Err(error.clone()),
            _ => Err(ApplicationError::Internal(
                "no mock response configured".to_string(),
            )),
        }
    }
}

fn build_client_with_llm(llm_client: MockLlmClient) -> Client {
    let llm_client: Arc<dyn LlmClient + Send + Sync> = Arc::new(llm_client);
    let rocket = api::build_rocket_with_llm_client(
        api::realtime::Hub::new(),
        NodeConfig { port: 53421 },
        llm_client,
    );
    Client::tracked(rocket).expect("valid rocket instance")
}

#[test]
fn can_build_rocket_with_mock_llm_client() {
    let _client = build_client_with_llm(MockLlmClient::default());
}

#[test]
fn llm_generate_returns_200_for_valid_request() {
    let response = GenerateContentResponse {
        candidates: vec![],
        usage_metadata: UsageMetadata::default(),
        ..Default::default()
    };
    let client = build_client_with_llm(MockLlmClient::new(response));

    let response = client
        .post("/api/llm/gemini")
        .header(ContentType::JSON)
        .body(json!({"contents": [{"parts": [{"text": "hello"}]}]}).to_string())
        .dispatch();

    assert_eq!(response.status(), Status::Ok);
    let body = response.into_string().expect("response body");
    let json: serde_json::Value = serde_json::from_str(&body).expect("valid json");
    assert!(json.get("body").is_some());
}

#[test]
fn llm_generate_returns_400_for_model_field() {
    let client = build_client_with_llm(MockLlmClient::default());

    let response = client
        .post("/api/llm/gemini")
        .header(ContentType::JSON)
        .body(
            json!({"model": "gemini-1.5-flash", "contents": [{"parts": [{"text": "hello"}]}]})
                .to_string(),
        )
        .dispatch();

    assert_eq!(response.status(), Status::BadRequest);
}

#[test]
fn llm_generate_returns_400_for_malformed_json() {
    let client = build_client_with_llm(MockLlmClient::default());

    let response = client
        .post("/api/llm/gemini")
        .header(ContentType::JSON)
        .body("{not valid json")
        .dispatch();

    assert_eq!(response.status(), Status::BadRequest);
    let body = response.into_string().expect("response body");
    let json: serde_json::Value = serde_json::from_str(&body).expect("valid json");
    assert!(json.get("body").is_some());
}

#[test]
fn llm_generate_returns_500_for_client_error() {
    let client = build_client_with_llm(MockLlmClient::with_error(ApplicationError::Internal(
        "GEMINI_API_KEY not set".to_string(),
    )));

    let response = client
        .post("/api/llm/gemini")
        .header(ContentType::JSON)
        .body(json!({"contents": [{"parts": [{"text": "hello"}]}]}).to_string())
        .dispatch();

    assert_eq!(response.status(), Status::InternalServerError);
}
