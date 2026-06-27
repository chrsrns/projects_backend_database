use super::{CustomJsonResult, UnauthorizedJsonResult};
use application::auth::{login, logout, me, register};
use domain::models::{AuthLoginRequest, AuthRegisterRequest, User};
use rocket::response::status::Unauthorized;
use rocket::serde::json::Json;
use rocket::{get, post};
use shared::response_models::{AuthTokenResponse, Response};

use crate::error::map_application_error;

#[utoipa::path(
    post,
    path = "/auth/register",
    tag = "Auth",
    request_body(content = AuthRegisterRequest, content_type = "application/json"),
    responses(
        (status = 201, description = "Created", body = Response<User>, content_type = "application/json"),
        (status = 409, description = "Conflict", body = Response<String>, content_type = "application/json")
    )
)]
#[post("/auth/register", format = "application/json", data = "<payload>")]
pub fn register_handler(payload: Json<AuthRegisterRequest>) -> CustomJsonResult<User> {
    match register::register(payload.into_inner()) {
        Ok(user) => Ok(rocket::response::status::Custom(
            rocket::http::Status::Created,
            Json(Response { body: user }),
        )),
        Err(err) => Err(map_application_error(err)),
    }
}

#[utoipa::path(
    post,
    path = "/auth/login",
    tag = "Auth",
    request_body(content = AuthLoginRequest, content_type = "application/json"),
    responses(
        (status = 200, description = "OK", body = Response<AuthTokenResponse>, content_type = "application/json"),
        (status = 401, description = "Unauthorized", body = Response<String>, content_type = "application/json")
    )
)]
#[post("/auth/login", format = "application/json", data = "<payload>")]
pub fn login_handler(payload: Json<AuthLoginRequest>) -> UnauthorizedJsonResult<AuthTokenResponse> {
    match login::login(payload.into_inner()) {
        Ok(token) => Ok(Json(Response { body: token })),
        Err(_) => Err(Unauthorized(Json(Response {
            body: "Invalid credentials".to_string(),
        }))),
    }
}

#[utoipa::path(
    get,
    path = "/auth/me",
    tag = "Auth",
    security(("bearerAuth" = [])),
    responses(
        (status = 200, description = "OK", body = Response<User>, content_type = "application/json"),
        (status = 401, description = "Unauthorized", body = Response<String>, content_type = "application/json")
    )
)]
#[get("/auth/me")]
pub fn me_handler(auth: crate::auth::AuthSession) -> UnauthorizedJsonResult<User> {
    match me::me(auth.user_id) {
        Ok(user) => Ok(Json(Response { body: user })),
        Err(_) => Err(Unauthorized(Json(Response {
            body: "Unauthorized".to_string(),
        }))),
    }
}

#[utoipa::path(
    post,
    path = "/auth/logout",
    tag = "Auth",
    security(("bearerAuth" = [])),
    responses(
        (status = 200, description = "OK", body = Response<String>, content_type = "application/json"),
        (status = 401, description = "Unauthorized", body = Response<String>, content_type = "application/json")
    )
)]
#[post("/auth/logout")]
pub fn logout_handler(auth: crate::auth::AuthSession) -> UnauthorizedJsonResult<String> {
    match logout::logout(auth.session_id) {
        Ok(()) => Ok(Json(Response {
            body: "Logged out".to_string(),
        })),
        Err(_) => Err(Unauthorized(Json(Response {
            body: "Unauthorized".to_string(),
        }))),
    }
}
