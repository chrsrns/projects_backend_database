use diesel::prelude::*;
use domain::models::{Resume, UpdateResume};
use infrastructure::establish_connection;

use crate::error::ApplicationError;

pub fn update_resume(
    user_id_value: i32,
    resume_id: i32,
    resume: UpdateResume,
) -> Result<Resume, ApplicationError> {
    use domain::schema::resumes;

    let existing: Resume = match resumes::table
        .find(resume_id)
        .first(&mut establish_connection())
    {
        Ok(r) => r,
        Err(diesel::result::Error::NotFound) => {
            return Err(ApplicationError::NotFound(format!(
                "Error updating resume with id {} - {}",
                resume_id,
                diesel::result::Error::NotFound
            )));
        }
        Err(err) => {
            return Err(ApplicationError::Internal(format!(
                "Database error - {}",
                err
            )));
        }
    };

    match existing.created_by {
        Some(owner) if owner == user_id_value => {}
        Some(_) | None => {
            return Err(ApplicationError::Forbidden);
        }
    }

    match diesel::update(resumes::table.find(resume_id))
        .set(&resume)
        .get_result::<Resume>(&mut establish_connection())
    {
        Ok(updated_resume) => Ok(updated_resume),
        Err(err) => match err {
            diesel::result::Error::NotFound => Err(ApplicationError::NotFound(format!(
                "Error updating resume with id {} - {}",
                resume_id, err
            ))),
            _ => Err(ApplicationError::Internal(format!(
                "Database error - {}",
                err
            ))),
        },
    }
}
