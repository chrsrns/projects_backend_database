use application::auth::{login, logout, me, register};
use application::error::ApplicationError;
use domain::models::{AuthLoginRequest, AuthRegisterRequest};
use rocket::response::status::{Custom, Unauthorized};
use rocket::serde::json::Json;
use rocket::{get, post};
use shared::response_models::{AuthTokenResponse, Response};

#[post("/auth/register", format = "application/json", data = "<payload>")]
pub fn register_handler(
    payload: Json<AuthRegisterRequest>,
) -> Result<Custom<Json<Response<domain::models::User>>>, Custom<Json<Response<String>>>> {
    match register::register(payload.into_inner()) {
        Ok(user) => Ok(Custom(
            rocket::http::Status::Created,
            Json(Response { body: user }),
        )),
        Err(ApplicationError::Conflict(msg)) => Err(Custom(
            rocket::http::Status::Conflict,
            Json(Response { body: msg }),
        )),
        Err(ApplicationError::BadRequest(msg)) => Err(Custom(
            rocket::http::Status::BadRequest,
            Json(Response { body: msg }),
        )),
        Err(ApplicationError::Unauthorized) => Err(Custom(
            rocket::http::Status::Unauthorized,
            Json(Response {
                body: "Unauthorized".to_string(),
            }),
        )),
        Err(ApplicationError::Internal(msg)) => Err(Custom(
            rocket::http::Status::InternalServerError,
            Json(Response { body: msg }),
        )),
        Err(ApplicationError::Forbidden) => Err(Custom(
            rocket::http::Status::Forbidden,
            Json(Response {
                body: "Forbidden".to_string(),
            }),
        )),
        Err(ApplicationError::NotFound(msg)) => Err(Custom(
            rocket::http::Status::NotFound,
            Json(Response { body: msg }),
        )),
    }
}

#[post("/auth/login", format = "application/json", data = "<payload>")]
pub fn login_handler(
    payload: Json<AuthLoginRequest>,
) -> Result<Json<Response<AuthTokenResponse>>, Unauthorized<Json<Response<String>>>> {
    match login::login(payload.into_inner()) {
        Ok(token) => Ok(Json(Response { body: token })),
        Err(_) => Err(Unauthorized(Json(Response {
            body: "Invalid credentials".to_string(),
        }))),
    }
}

#[get("/auth/me")]
pub fn me_handler(
    auth: crate::auth::AuthSession,
) -> Result<Json<Response<domain::models::User>>, Unauthorized<Json<Response<String>>>> {
    match me::me(auth.user_id) {
        Ok(user) => Ok(Json(Response { body: user })),
        Err(_) => Err(Unauthorized(Json(Response {
            body: "Unauthorized".to_string(),
        }))),
    }
}

#[post("/auth/logout")]
pub fn logout_handler(
    auth: crate::auth::AuthSession,
) -> Result<Json<Response<String>>, Unauthorized<Json<Response<String>>>> {
    match logout::logout(auth.session_id) {
        Ok(()) => Ok(Json(Response {
            body: "Logged out".to_string(),
        })),
        Err(_) => Err(Unauthorized(Json(Response {
            body: "Unauthorized".to_string(),
        }))),
    }
}
