use application::auth::{login, logout, me, register};
use application::error::ApplicationError;
use domain::models::{AuthLoginRequest, AuthRegisterRequest, User};
use rocket::response::status::{Custom, Unauthorized};
use rocket::serde::json::Json;
use rocket::{get, post};
use shared::response_models::{AuthTokenResponse, Response};

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
pub fn register_handler(
    payload: Json<AuthRegisterRequest>,
) -> Result<Custom<Json<Response<User>>>, Custom<Json<Response<String>>>> {
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
pub fn me_handler(
    auth: crate::auth::AuthSession,
) -> Result<Json<Response<User>>, Unauthorized<Json<Response<String>>>> {
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
