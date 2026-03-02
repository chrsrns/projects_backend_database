use argon2::{Argon2, PasswordHasher, password_hash::SaltString};
use diesel::prelude::*;
use domain::models::{AuthRegisterRequest, NewUser, User};
use infrastructure::establish_connection;
use rand_core::OsRng;
use rocket::response::status::{Conflict, Created};
use rocket::serde::json::Json;
use shared::response_models::Response;

pub fn register(payload: Json<AuthRegisterRequest>) -> Result<Created<String>, Conflict<String>> {
    use domain::schema::users;

    let payload = payload.into_inner();

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(payload.password.as_bytes(), &salt)
        .map_err(|_| {
            let response = Response::<String> {
                body: "Password hashing failed".to_string(),
            };
            Conflict(serde_json::to_string(&response).unwrap())
        })?
        .to_string();

    let new_user = NewUser {
        email: payload.email,
        password_hash,
    };

    match diesel::insert_into(users::table)
        .values(&new_user)
        .get_result::<User>(&mut establish_connection())
    {
        Ok(user) => {
            let response = Response::<User> { body: user };
            Ok(Created::new("").tagged_body(serde_json::to_string(&response).unwrap()))
        }
        Err(err) => match err {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UniqueViolation,
                _,
            ) => {
                let response = Response::<String> {
                    body: "User with this email already exists".to_string(),
                };
                Err(Conflict(serde_json::to_string(&response).unwrap()))
            }
            _ => panic!("Database error - {}", err),
        },
    }
}
