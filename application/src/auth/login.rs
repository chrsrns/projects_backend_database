use argon2::{Argon2, PasswordHash, PasswordVerifier};
use chrono::{Duration, Utc};
use diesel::prelude::*;
use domain::models::{AuthLoginRequest, NewSession, Session, User};
use infrastructure::establish_connection;
use shared::response_models::AuthTokenResponse;

use crate::error::ApplicationError;

pub fn login(payload: AuthLoginRequest) -> Result<AuthTokenResponse, ApplicationError> {
    use domain::schema::sessions;
    use domain::schema::users::dsl::*;

    let user: User = match users
        .filter(email.eq(payload.email))
        .first::<User>(&mut establish_connection())
    {
        Ok(u) => u,
        Err(_) => return Err(ApplicationError::Unauthorized),
    };

    let parsed_hash =
        PasswordHash::new(&user.password_hash).map_err(|_| ApplicationError::Unauthorized)?;

    Argon2::default()
        .verify_password(payload.password.as_bytes(), &parsed_hash)
        .map_err(|_| ApplicationError::Unauthorized)?;

    let expires_at = (Utc::now() + Duration::days(1)).naive_utc();

    let new_session = NewSession {
        user_id: user.id,
        expires_at,
    };

    let session: Session = diesel::insert_into(sessions::table)
        .values(&new_session)
        .get_result::<Session>(&mut establish_connection())
        .map_err(|_| ApplicationError::Internal("Failed to create session".to_string()))?;

    Ok(AuthTokenResponse {
        token: session.id.to_string(),
        expires_at: session.expires_at,
    })
}
