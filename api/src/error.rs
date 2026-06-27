use application::error::ApplicationError;
use rocket::http::Status;
use rocket::response::status::Custom;
use rocket::serde::json::Json;
use shared::response_models::Response;

/// Maps an [`ApplicationError`] to a Rocket [`Custom`] JSON response.
///
/// `Internal` errors are logged server-side and returned to the client as a
/// generic `"Internal server error"` message so that internal details (table
/// names, query text, etc.) are never exposed (V13).
pub fn map_application_error(err: ApplicationError) -> Custom<Json<Response<String>>> {
    match err {
        ApplicationError::NotFound(msg) => {
            Custom(Status::NotFound, Json(Response { body: msg }))
        }
        ApplicationError::Forbidden => Custom(
            Status::Forbidden,
            Json(Response {
                body: "Forbidden".to_string(),
            }),
        ),
        ApplicationError::Unauthorized => Custom(
            Status::Unauthorized,
            Json(Response {
                body: "Unauthorized".to_string(),
            }),
        ),
        ApplicationError::Conflict(msg) => {
            Custom(Status::Conflict, Json(Response { body: msg }))
        }
        ApplicationError::BadRequest(msg) => {
            Custom(Status::BadRequest, Json(Response { body: msg }))
        }
        ApplicationError::Internal(msg) => {
            log::error!("Internal server error: {}", msg);
            Custom(
                Status::InternalServerError,
                Json(Response {
                    body: "Internal server error".to_string(),
                }),
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_internal_error_hides_details() {
        let response = map_application_error(ApplicationError::Internal(
            "Database error - secret table name".to_string(),
        ));
        assert_eq!(response.0, Status::InternalServerError);
        assert_eq!(response.1.0.body, "Internal server error");
    }

    #[test]
    fn test_not_found_returns_message() {
        let response =
            map_application_error(ApplicationError::NotFound("Resume not found".to_string()));
        assert_eq!(response.0, Status::NotFound);
        assert_eq!(response.1.0.body, "Resume not found");
    }

    #[test]
    fn test_bad_request_returns_message() {
        let response = map_application_error(ApplicationError::BadRequest(
            "Invalid email".to_string(),
        ));
        assert_eq!(response.0, Status::BadRequest);
        assert_eq!(response.1.0.body, "Invalid email");
    }

    #[test]
    fn test_forbidden_returns_generic_message() {
        let response = map_application_error(ApplicationError::Forbidden);
        assert_eq!(response.0, Status::Forbidden);
        assert_eq!(response.1.0.body, "Forbidden");
    }

    #[test]
    fn test_unauthorized_returns_generic_message() {
        let response = map_application_error(ApplicationError::Unauthorized);
        assert_eq!(response.0, Status::Unauthorized);
        assert_eq!(response.1.0.body, "Unauthorized");
    }

    #[test]
    fn test_conflict_returns_message() {
        let response =
            map_application_error(ApplicationError::Conflict("Unique violation".to_string()));
        assert_eq!(response.0, Status::Conflict);
        assert_eq!(response.1.0.body, "Unique violation");
    }
}
