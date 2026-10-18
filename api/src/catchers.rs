use rocket::http::Status;
use rocket::response::status::Custom;
use rocket::serde::json::Json;
use shared::response_models::Response;

// Framework-level rejections under `/api` keep the `Response<String>`
// envelope instead of Rocket's default HTML catcher (V250). Messages are
// fixed per status — never request-derived.
fn envelope(status: Status, message: &str) -> Custom<Json<Response<String>>> {
    Custom(
        status,
        Json(Response {
            body: message.to_string(),
        }),
    )
}

#[catch(400)]
pub fn bad_request() -> Custom<Json<Response<String>>> {
    envelope(Status::BadRequest, "Bad request")
}

#[catch(401)]
pub fn unauthorized() -> Custom<Json<Response<String>>> {
    envelope(Status::Unauthorized, "Unauthorized")
}

#[catch(413)]
pub fn payload_too_large() -> Custom<Json<Response<String>>> {
    envelope(Status::PayloadTooLarge, "Payload too large")
}

#[catch(415)]
pub fn unsupported_media_type() -> Custom<Json<Response<String>>> {
    envelope(Status::UnsupportedMediaType, "Unsupported media type")
}

#[catch(422)]
pub fn unprocessable_entity() -> Custom<Json<Response<String>>> {
    envelope(Status::UnprocessableEntity, "Unprocessable entity")
}

#[catch(500)]
pub fn internal_error() -> Custom<Json<Response<String>>> {
    envelope(Status::InternalServerError, "Internal server error")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_catcher_emits_a_fixed_envelope() {
        for (catcher, status) in [
            (bad_request(), Status::BadRequest),
            (unauthorized(), Status::Unauthorized),
            (payload_too_large(), Status::PayloadTooLarge),
            (unsupported_media_type(), Status::UnsupportedMediaType),
            (unprocessable_entity(), Status::UnprocessableEntity),
            (internal_error(), Status::InternalServerError),
        ] {
            assert_eq!(catcher.0, status);
            assert!(
                !catcher.1.0.body.is_empty(),
                "{status} message must be fixed and non-empty"
            );
        }
    }
}
