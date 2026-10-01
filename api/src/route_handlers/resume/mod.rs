pub type JsonResult<T> = Result<
    rocket::serde::json::Json<shared::response_models::Response<T>>,
    rocket::response::status::Custom<
        rocket::serde::json::Json<shared::response_models::Response<String>>,
    >,
>;

pub type CustomJsonResult<T> = Result<
    rocket::response::status::Custom<
        rocket::serde::json::Json<shared::response_models::Response<T>>,
    >,
    rocket::response::status::Custom<
        rocket::serde::json::Json<shared::response_models::Response<String>>,
    >,
>;

pub type ConflictJsonResult<T> = Result<
    rocket::response::status::Custom<
        rocket::serde::json::Json<shared::response_models::Response<T>>,
    >,
    rocket::response::status::Conflict<
        rocket::serde::json::Json<shared::response_models::Response<String>>,
    >,
>;

pub type NoContentResult = Result<
    rocket::response::status::NoContent,
    rocket::response::status::Custom<
        rocket::serde::json::Json<shared::response_models::Response<String>>,
    >,
>;

pub type UnauthorizedJsonResult<T> = Result<
    rocket::serde::json::Json<shared::response_models::Response<T>>,
    rocket::response::status::Unauthorized<
        rocket::serde::json::Json<shared::response_models::Response<String>>,
    >,
>;

pub mod auth_handler;
pub mod education_handler;
pub mod frameworks_handler;
pub mod languages_handler;
pub mod markdown_handler;
pub mod portfolio_projects_handler;
pub mod resume_handler;
pub mod skills_handler;
pub mod variant_handler;
pub mod work_experiences_handler;
