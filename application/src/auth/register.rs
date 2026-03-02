use crate::error::ApplicationError;
use argon2::{Argon2, PasswordHasher, password_hash::SaltString};
use diesel::prelude::*;
use domain::models::{AuthRegisterRequest, NewUser, User};
use infrastructure::establish_connection;
use rand_core::OsRng;

pub fn register(payload: AuthRegisterRequest) -> Result<User, ApplicationError> {
    use domain::schema::users;

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(payload.password.as_bytes(), &salt)
        .map_err(|_| ApplicationError::Internal("Password hashing failed".to_string()))?
        .to_string();

    let new_user = NewUser {
        email: payload.email,
        password_hash,
    };

    match diesel::insert_into(users::table)
        .values(&new_user)
        .get_result::<User>(&mut establish_connection())
    {
        Ok(user) => Ok(user),
        Err(err) => match err {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UniqueViolation,
                _,
            ) => Err(ApplicationError::Conflict(
                "User with this email already exists".to_string(),
            )),
            _ => Err(ApplicationError::Internal(format!(
                "Database error - {}",
                err
            ))),
        },
    }
}
