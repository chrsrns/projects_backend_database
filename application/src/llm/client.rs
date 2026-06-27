use crate::error::ApplicationError;
use crate::llm::types::{GenerateContentRequest, GenerateContentResponse};

#[rocket::async_trait]
pub trait LlmClient: Send + Sync {
    async fn generate_content(
        &self,
        request: GenerateContentRequest,
    ) -> Result<GenerateContentResponse, ApplicationError>;
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[derive(Default)]
    pub struct MockLlmClient {
        response: Option<GenerateContentResponse>,
        error: Option<ApplicationError>,
    }

    impl MockLlmClient {
        pub fn new(response: GenerateContentResponse) -> Self {
            Self {
                response: Some(response),
                error: None,
            }
        }

        pub fn with_error(error: ApplicationError) -> Self {
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
}
