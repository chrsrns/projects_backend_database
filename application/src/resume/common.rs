use domain::models::Resume;

use crate::error::ApplicationError;

pub fn find_resume(resume_id: i32) -> Result<Resume, ApplicationError> {
    use diesel::prelude::{QueryDsl, RunQueryDsl};
    use domain::schema::resumes;
    use infrastructure::establish_connection;

    let existing: Resume = match resumes::table
        .find(resume_id)
        .first(&mut establish_connection())
    {
        Ok(r) => r,
        Err(err) => return Err(app_err_from_diesel_err(err)),
    };
    Ok(existing)
}

pub fn app_err_from_diesel_err(err: diesel::result::Error) -> ApplicationError {
    match err {
        diesel::result::Error::DatabaseError(
            diesel::result::DatabaseErrorKind::UniqueViolation,
            _,
        ) => ApplicationError::Conflict("Unique violation".to_string()),
        diesel::result::Error::NotFound => ApplicationError::NotFound("Not found".to_string()),
        _ => ApplicationError::Internal(format!("Database error - {}", err)),
    }
}
