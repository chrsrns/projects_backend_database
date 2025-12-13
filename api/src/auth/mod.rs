use rocket::Request;
use rocket::http::Status;
use rocket::request::{FromRequest, Outcome};

#[derive(Debug, Clone)]
pub struct AuthSession {
    pub session_id: uuid::Uuid,
    pub user_id: i32,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthSession {
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let auth_header = match req.headers().get_one("Authorization") {
            Some(v) => v,
            None => return Outcome::Error((Status::Unauthorized, ())),
        };

        let token = auth_header
            .strip_prefix("Bearer ")
            .or_else(|| auth_header.strip_prefix("bearer "));

        let token = match token {
            Some(t) => t,
            None => return Outcome::Error((Status::Unauthorized, ())),
        };

        let session_id = match token.parse::<uuid::Uuid>() {
            Ok(v) => v,
            Err(_) => return Outcome::Error((Status::Unauthorized, ())),
        };

        match application::auth::me::resolve_session_user_id(session_id) {
            Ok(Some(user_id)) => Outcome::Success(AuthSession {
                session_id,
                user_id,
            }),
            Ok(None) => Outcome::Error((Status::Unauthorized, ())),
            Err(_) => Outcome::Error((Status::Unauthorized, ())),
        }
    }
}
