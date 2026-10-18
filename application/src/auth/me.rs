use diesel::prelude::*;
use domain::models::User;
use infrastructure::establish_connection;
use uuid::Uuid;

use crate::error::ApplicationError;

/// Outcome of looking a session token up in the `sessions` table.
///
/// The states are deliberately distinct. `Absent` means the token names no
/// unexpired session and is a normal answer. `Failed` means the lookup
/// itself could not run, and must never be read as absence: downgrading a
/// database error to "no session" would silently serve anonymous content in
/// place of an authorized read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionLookup {
    Resolved(i32),
    Absent,
    Failed,
}

/// Classifies a session lookup result. Pure, so the failure branch is
/// testable without a database or an HTTP request.
pub fn classify_session_lookup(
    result: Result<Option<i32>, diesel::result::Error>,
) -> SessionLookup {
    match result {
        Ok(Some(user_id)) => SessionLookup::Resolved(user_id),
        Ok(None) => SessionLookup::Absent,
        Err(_) => SessionLookup::Failed,
    }
}

pub fn resolve_session(session_id: Uuid) -> SessionLookup {
    classify_session_lookup(resolve_session_user_id(session_id))
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_session_lookup_resolves_a_user() {
        assert_eq!(
            classify_session_lookup(Ok(Some(7))),
            SessionLookup::Resolved(7)
        );
    }

    #[test]
    fn classify_session_lookup_treats_a_missing_row_as_absent() {
        assert_eq!(classify_session_lookup(Ok(None)), SessionLookup::Absent);
    }

    #[test]
    fn classify_session_lookup_never_reports_a_lookup_error_as_absent() {
        assert_eq!(
            classify_session_lookup(Err(diesel::result::Error::NotFound)),
            SessionLookup::Failed
        );
        assert_eq!(
            classify_session_lookup(Err(diesel::result::Error::RollbackTransaction)),
            SessionLookup::Failed
        );
    }
}
