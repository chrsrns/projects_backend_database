use diesel::prelude::{BoolExpressionMethods, ExpressionMethods, QueryDsl, RunQueryDsl};
use domain::models::Resume;
use infrastructure::establish_connection;

use crate::error::ApplicationError;

pub fn find_resume(resume_id: i32) -> Result<Resume, ApplicationError> {
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

pub fn find_accessible_resume(
    resume_id_value: i32,
    user_id_value: Option<i32>,
) -> Result<Resume, ApplicationError> {
    use domain::schema::resumes::dsl as resumes_dsl;

    let mut resume_query = resumes_dsl::resumes.into_boxed();
    resume_query = resume_query.filter(resumes_dsl::id.eq(resume_id_value));
    resume_query = match user_id_value {
        Some(uid) => resume_query.filter(
            resumes_dsl::is_public
                .eq(true)
                .or(resumes_dsl::created_by.eq(uid)),
        ),
        None => resume_query.filter(resumes_dsl::is_public.eq(true)),
    };

    match resume_query.first(&mut establish_connection()) {
        Ok(r) => Ok(r),
        Err(diesel::result::Error::NotFound) => Err(ApplicationError::NotFound(format!(
            "Resume with id {} not found",
            resume_id_value
        ))),
        Err(err) => Err(app_err_from_diesel_err(err)),
    }
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
