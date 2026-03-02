use diesel::prelude::*;
use infrastructure::establish_connection;
use uuid::Uuid;

use crate::error::ApplicationError;

pub fn logout(session_id_value: Uuid) -> Result<(), ApplicationError> {
    use domain::schema::sessions::dsl::*;

    match diesel::delete(sessions.filter(id.eq(session_id_value)))
        .execute(&mut establish_connection())
    {
        Ok(count) => {
            if count == 0 {
                Err(ApplicationError::Unauthorized)
            } else {
                Ok(())
            }
        }
        Err(err) => Err(ApplicationError::Internal(format!(
            "Database error - {}",
            err
        ))),
    }
}
