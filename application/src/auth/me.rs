use diesel::prelude::*;
use domain::models::User;
use infrastructure::establish_connection;
use uuid::Uuid;

use crate::error::ApplicationError;

pub fn resolve_session_user_id(session_id: Uuid) -> Result<Option<i32>, diesel::result::Error> {
    use domain::schema::sessions::dsl::*;

    sessions
        .select(user_id)
        .filter(id.eq(session_id))
        .filter(expires_at.gt(chrono::Utc::now().naive_utc()))
        .first::<i32>(&mut establish_connection())
        .optional()
}

pub fn me(user_id_value: i32) -> Result<User, ApplicationError> {
    use domain::schema::users::dsl::*;

    let user: User = match users
        .find(user_id_value)
        .first::<User>(&mut establish_connection())
    {
        Ok(u) => u,
        Err(_) => return Err(ApplicationError::Unauthorized),
    };

    Ok(user)
}
