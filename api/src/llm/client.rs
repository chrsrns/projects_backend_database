use application::error::ApplicationError;
use application::llm::{GenerateContentRequest, GenerateContentResponse, LlmClient};

pub struct GeminiClient {
    api_key: Option<String>,
    model: String,
    inner: gemini_client_rs::GeminiClient,
}

impl GeminiClient {
    pub fn new(api_key: Option<String>, model: String) -> Self {
        let inner = api_key
            .as_ref()
            .map(|key| gemini_client_rs::GeminiClient::new(key.clone()))
            .unwrap_or_default();
        Self {
            api_key,
            model,
            inner,
        }
    }
}

fn to_sdk_request(
    request: GenerateContentRequest,
) -> Result<gemini_client_rs::types::GenerateContentRequest, ApplicationError> {
    let value = serde_json::to_value(request)
        .map_err(|e| ApplicationError::Internal(format!("LLM request serialization error: {e}")))?;
    serde_json::from_value(value)
        .map_err(|e| ApplicationError::Internal(format!("LLM request shape mismatch: {e}")))
}

fn from_sdk_response(
    response: gemini_client_rs::types::GenerateContentResponse,
) -> Result<GenerateContentResponse, ApplicationError> {
    let value = serde_json::to_value(response).map_err(|e| {
        ApplicationError::Internal(format!("LLM response serialization error: {e}"))
    })?;
    serde_json::from_value(value)
        .map_err(|e| ApplicationError::Internal(format!("LLM response shape mismatch: {e}")))
}

#[rocket::async_trait]
impl LlmClient for GeminiClient {
    async fn generate_content(
        &self,
        request: GenerateContentRequest,
    ) -> Result<GenerateContentResponse, ApplicationError> {
        if self.api_key.is_none() {
            return Err(ApplicationError::Internal(
                "GEMINI_API_KEY not set".to_string(),
            ));
        }

        let sdk_request = to_sdk_request(request)?;
        let sdk_response = self
            .inner
            .generate_content(&self.model, &sdk_request)
            .await
            .map_err(|e| ApplicationError::Internal(format!("Gemini API error: {e}")))?;

        from_sdk_response(sdk_response)
    }
}
