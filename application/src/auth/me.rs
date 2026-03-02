use diesel::prelude::*;
use domain::models::User;
use infrastructure::establish_connection;
use rocket::response::status::Unauthorized;
use shared::response_models::Response;
use uuid::Uuid;

pub fn resolve_session_user_id(session_id: Uuid) -> Result<Option<i32>, diesel::result::Error> {
    use domain::schema::sessions::dsl::*;

    sessions
        .select(user_id)
        .filter(id.eq(session_id))
        .filter(expires_at.gt(chrono::Utc::now().naive_utc()))
        .first::<i32>(&mut establish_connection())
        .optional()
}

pub fn me(user_id_value: i32) -> Result<String, Unauthorized<String>> {
    use domain::schema::users::dsl::*;

    let user: User = match users
        .find(user_id_value)
        .first::<User>(&mut establish_connection())
    {
        Ok(u) => u,
        Err(_) => {
            let response = Response::<String> {
                body: "Unauthorized".to_string(),
            };
            return Err(Unauthorized(serde_json::to_string(&response).unwrap()));
        }
    };

    let response = Response::<User> { body: user };

    Ok(serde_json::to_string(&response).unwrap())
}
