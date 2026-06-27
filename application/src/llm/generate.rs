use crate::error::ApplicationError;
use crate::llm::client::LlmClient;
use crate::llm::types::{GenerateContentRequest, GenerateContentResponse};

pub async fn generate_content(
    client: &dyn LlmClient,
    request: GenerateContentRequest,
) -> Result<GenerateContentResponse, ApplicationError> {
    client.generate_content(request).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::client::tests::MockLlmClient;
    use crate::llm::types::UsageMetadata;

    #[tokio::test]
    async fn generate_content_delegates_to_client() {
        let expected = GenerateContentResponse {
            candidates: vec![],
            usage_metadata: UsageMetadata::default(),
            ..Default::default()
        };
        let client = MockLlmClient::new(expected.clone());
        let result = generate_content(&client, GenerateContentRequest::default()).await;
        assert_eq!(result.unwrap(), expected);
    }

    #[tokio::test]
    async fn generate_content_returns_client_error() {
        let client =
            MockLlmClient::with_error(ApplicationError::Internal("gemini unavailable".to_string()));
        let result = generate_content(&client, GenerateContentRequest::default()).await;
        assert_eq!(
            result,
            Err(ApplicationError::Internal("gemini unavailable".to_string()))
        );
    }
}
