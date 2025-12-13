use application::auth::{login, logout, me, register};
use domain::models::{AuthLoginRequest, AuthRegisterRequest};
use rocket::response::status::{Conflict, Created, Unauthorized};
use rocket::serde::json::Json;
use rocket::{get, post};

#[post("/auth/register", format = "application/json", data = "<payload>")]
pub fn register_handler(
    payload: Json<AuthRegisterRequest>,
) -> Result<Created<String>, Conflict<String>> {
    register::register(payload)
}

#[post("/auth/login", format = "application/json", data = "<payload>")]
pub fn login_handler(payload: Json<AuthLoginRequest>) -> Result<String, Unauthorized<String>> {
    login::login(payload)
}

#[get("/auth/me")]
pub fn me_handler(auth: crate::auth::AuthSession) -> Result<String, Unauthorized<String>> {
    me::me(auth.user_id)
}

#[post("/auth/logout")]
pub fn logout_handler(auth: crate::auth::AuthSession) -> Result<String, Unauthorized<String>> {
    logout::logout(auth.session_id)
}
