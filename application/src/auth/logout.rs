use diesel::prelude::*;
use infrastructure::establish_connection;
use rocket::response::status::Unauthorized;
use shared::response_models::Response;
use uuid::Uuid;

pub fn logout(session_id_value: Uuid) -> Result<String, Unauthorized<String>> {
    use domain::schema::sessions::dsl::*;

    match diesel::delete(sessions.filter(id.eq(session_id_value)))
        .execute(&mut establish_connection())
    {
        Ok(count) => {
            if count == 0 {
                let response = Response::<String> {
                    body: "Unauthorized".to_string(),
                };
                Err(Unauthorized(serde_json::to_string(&response).unwrap()))
            } else {
                let response = Response::<String> {
                    body: "Logged out".to_string(),
                };
                Ok(serde_json::to_string(&response).unwrap())
            }
        }
        Err(_) => {
            let response = Response::<String> {
                body: "Unauthorized".to_string(),
            };
            Err(Unauthorized(serde_json::to_string(&response).unwrap()))
        }
    }
}
