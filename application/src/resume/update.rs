use diesel::prelude::*;
use domain::models::{Resume, UpdateResume};
use domain::schema::resumes;
use infrastructure::establish_connection;

use crate::error::ApplicationError;
use crate::resume::common::{app_err_from_diesel_err, find_resume};

pub fn update_resume(
    user_id_value: i32,
    resume_id: i32,
    resume: UpdateResume,
) -> Result<Resume, ApplicationError> {
    let existing = match find_resume(resume_id) {
        Ok(value) => value,
        Err(value) => return Err(value),
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
        Err(err) => Err(app_err_from_diesel_err(err)),
    }
}
