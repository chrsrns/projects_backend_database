use rocket::Request;
use rocket::http::Status;
use rocket::request::{FromRequest, Outcome};
use uuid::Uuid;

use application::auth::me::{SessionLookup, resolve_session};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthSession {
    pub session_id: Uuid,
    pub user_id: i32,
}

#[derive(Debug, Clone)]
pub struct MaybeAuthSession(pub Option<AuthSession>);

/// How a bearer decodes before any database lookup.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BearerToken {
    /// No `Authorization` header at all.
    Missing,
    /// A header that carries no usable token: an unknown scheme, a missing
    /// prefix, or a token that is not a UUID.
    Malformed,
    Parsed(Uuid),
}

/// Decodes an `Authorization` header value. Both `Bearer ` and lowercase
/// `bearer ` prefixes are accepted.
pub fn parse_bearer_token(header: Option<&str>) -> BearerToken {
    let Some(header) = header else {
        return BearerToken::Missing;
    };

    match header
        .strip_prefix("Bearer ")
        .or_else(|| header.strip_prefix("bearer "))
    {
        Some(token) => parse_token(token),
        None => BearerToken::Malformed,
    }
}

/// Decodes a bare session token, as sent in a WebSocket subscribe message.
pub fn parse_token(token: &str) -> BearerToken {
    match token.parse::<Uuid>() {
        Ok(session_id) => BearerToken::Parsed(session_id),
        Err(_) => BearerToken::Malformed,
    }
}

/// A caller's bearer resolved into a session.
///
/// `Anonymous` and `Failed` are kept apart on purpose. An unresolvable
/// bearer is a normal answer that leaves the request anonymous, whereas a
/// failed lookup must not be downgraded to anonymous — an authorized read
/// would silently become a public one — and must not answer 401 either,
/// because the client treats 401 as proof its session is dead.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionResolution {
    Anonymous,
    Authenticated(AuthSession),
    Failed,
}

impl SessionResolution {
    fn from_parsed(parsed: BearerToken) -> Self {
        let session_id = match parsed {
            BearerToken::Parsed(session_id) => session_id,
            BearerToken::Missing | BearerToken::Malformed => return Self::Anonymous,
        };

        match resolve_session(session_id) {
            SessionLookup::Resolved(user_id) => Self::Authenticated(AuthSession {
                session_id,
                user_id,
            }),
            SessionLookup::Absent => Self::Anonymous,
            SessionLookup::Failed => Self::Failed,
        }
    }
}

/// Resolves an `Authorization` header value into a session.
pub fn resolve_bearer(header: Option<&str>) -> SessionResolution {
    SessionResolution::from_parsed(parse_bearer_token(header))
}

/// Resolves a bare session token into a session.
pub fn resolve_token(token: Option<&str>) -> SessionResolution {
    SessionResolution::from_parsed(match token {
        Some(token) => parse_token(token),
        None => BearerToken::Missing,
    })
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthSession {
    type Error = ();

    /// Strict: a route that requires a session answers 401 for every bearer
    /// it cannot resolve, a failed lookup included.
    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        match resolve_bearer(req.headers().get_one("Authorization")) {
            SessionResolution::Authenticated(session) => Outcome::Success(session),
            SessionResolution::Anonymous | SessionResolution::Failed => {
                Outcome::Error((Status::Unauthorized, ()))
            }
        }
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for MaybeAuthSession {
    type Error = ();

    /// Token-optional: a bearer that names no unexpired session reads as
    /// absent, so the handler runs with anonymous semantics. A failed lookup
    /// fails closed with a 500 instead of impersonating an anonymous caller.
    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        match resolve_bearer(req.headers().get_one("Authorization")) {
            SessionResolution::Authenticated(session) => {
                Outcome::Success(MaybeAuthSession(Some(session)))
            }
            SessionResolution::Anonymous => Outcome::Success(MaybeAuthSession(None)),
            SessionResolution::Failed => Outcome::Error((Status::InternalServerError, ())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_header_decodes_as_missing() {
        assert_eq!(parse_bearer_token(None), BearerToken::Missing);
    }

    #[test]
    fn both_bearer_prefixes_decode_to_the_same_session() {
        let session_id = Uuid::new_v4();

        assert_eq!(
            parse_bearer_token(Some(&format!("Bearer {session_id}"))),
            BearerToken::Parsed(session_id)
        );
        assert_eq!(
            parse_bearer_token(Some(&format!("bearer {session_id}"))),
            BearerToken::Parsed(session_id)
        );
    }

    #[test]
    fn header_without_a_known_prefix_is_malformed() {
        let session_id = Uuid::new_v4();

        assert_eq!(
            parse_bearer_token(Some(&session_id.to_string())),
            BearerToken::Malformed
        );
        assert_eq!(
            parse_bearer_token(Some(&format!("Basic {session_id}"))),
            BearerToken::Malformed
        );
    }

    #[test]
    fn token_that_is_not_a_uuid_is_malformed() {
        assert_eq!(
            parse_bearer_token(Some("Bearer not-a-uuid")),
            BearerToken::Malformed
        );
        assert_eq!(parse_token("not-a-uuid"), BearerToken::Malformed);
    }

    #[test]
    fn bare_token_decodes_when_it_is_a_uuid() {
        let session_id = Uuid::new_v4();

        assert_eq!(
            parse_token(&session_id.to_string()),
            BearerToken::Parsed(session_id)
        );
    }

    #[test]
    fn empty_and_absent_tokens_read_as_anonymous() {
        assert_eq!(parse_token(""), BearerToken::Malformed);
        assert_eq!(resolve_token(None), SessionResolution::Anonymous);
    }
}
