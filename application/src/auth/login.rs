use argon2::{Argon2, PasswordHash, PasswordVerifier};
use chrono::{Duration, Utc};
use diesel::prelude::*;
use domain::models::{AuthLoginRequest, NewSession, Session, User};
use infrastructure::establish_connection;
use rocket::response::status::Unauthorized;
use rocket::serde::json::Json;
use shared::response_models::{AuthTokenResponse, Response, ResponseBody};

pub fn login(payload: Json<AuthLoginRequest>) -> Result<String, Unauthorized<String>> {
    use domain::schema::sessions;
    use domain::schema::users::dsl::*;

    let payload = payload.into_inner();

    let user: User = match users
        .filter(email.eq(payload.email))
        .first::<User>(&mut establish_connection())
    {
        Ok(u) => u,
        Err(_) => {
            let response = Response {
                body: ResponseBody::Message("Invalid credentials".to_string()),
            };
            return Err(Unauthorized(serde_json::to_string(&response).unwrap()));
        }
    };

    let parsed_hash = PasswordHash::new(&user.password_hash).map_err(|_| {
        let response = Response {
            body: ResponseBody::Message("Invalid credentials".to_string()),
        };
        Unauthorized(serde_json::to_string(&response).unwrap())
    })?;

    Argon2::default()
        .verify_password(payload.password.as_bytes(), &parsed_hash)
        .map_err(|_| {
            let response = Response {
                body: ResponseBody::Message("Invalid credentials".to_string()),
            };
            Unauthorized(serde_json::to_string(&response).unwrap())
        })?;

    let expires_at = (Utc::now() + Duration::days(1)).naive_utc();

    let new_session = NewSession {
        user_id: user.id,
        expires_at,
    };

    let session: Session = diesel::insert_into(sessions::table)
        .values(&new_session)
        .get_result::<Session>(&mut establish_connection())
        .map_err(|_| {
            let response = Response {
                body: ResponseBody::Message("Failed to create session".to_string()),
            };
            Unauthorized(serde_json::to_string(&response).unwrap())
        })?;

    let response = Response {
        body: ResponseBody::AuthToken(AuthTokenResponse {
            token: session.id.to_string(),
            expires_at: session.expires_at,
        }),
    };

    Ok(serde_json::to_string(&response).unwrap())
}
