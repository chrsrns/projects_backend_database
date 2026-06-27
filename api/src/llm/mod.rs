pub mod client;
pub mod handler;

pub use application::llm::LlmClient;
pub use client::GeminiClient;
pub use handler::generate_handler;

pub type CustomJsonResult<T> = Result<
    rocket::response::status::Custom<
        rocket::serde::json::Json<shared::response_models::Response<T>>,
    >,
    rocket::response::status::Custom<
        rocket::serde::json::Json<shared::response_models::Response<String>>,
    >,
>;
