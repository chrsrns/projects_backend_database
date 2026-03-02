use diesel::prelude::*;
use domain::models::{NewResume, NewResumeRequest, Resume};
use infrastructure::establish_connection;

use crate::error::ApplicationError;

pub fn create_resume(
    user_id_value: i32,
    resume: NewResumeRequest,
) -> Result<Resume, ApplicationError> {
    use domain::schema::resumes;

    let new_resume = NewResume {
        name: resume.name,
        profile_image_url: resume.profile_image_url,
        location: resume.location,
        email: resume.email,
        github_url: resume.github_url,
        mobile_number: resume.mobile_number,
        created_by: Some(user_id_value),
        is_public: resume.is_public.unwrap_or(false),
    };

    match diesel::insert_into(resumes::table)
        .values(&new_resume)
        .get_result::<Resume>(&mut establish_connection())
    {
        Ok(resume) => Ok(resume),
        Err(err) => match err {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UniqueViolation,
                _,
            ) => Err(ApplicationError::Conflict(
                "Resume with this email already exists".to_string(),
            )),
            _ => Err(ApplicationError::Internal(format!(
                "Database error - {}",
                err
            ))),
        },
    }
}
