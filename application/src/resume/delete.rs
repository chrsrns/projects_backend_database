use diesel::prelude::*;
use domain::models::Resume;
use infrastructure::establish_connection;

use crate::{error::ApplicationError, resume::common::app_err_from_diesel_err};

pub fn delete_resume(user_id_value: i32, resume_id: i32) -> Result<(), ApplicationError> {
    use domain::schema::resumes;

    let existing: Resume = match resumes::table
        .find(resume_id)
        .first(&mut establish_connection())
    {
        Ok(r) => r,
        Err(err) => {
            return Err(app_err_from_diesel_err(err));
        }
    };

    match existing.created_by {
        Some(owner) if owner == user_id_value => {}
        Some(_) | None => {
            return Err(ApplicationError::Forbidden);
        }
    }

    match diesel::delete(resumes::table.find(resume_id)).execute(&mut establish_connection()) {
        Ok(count) => {
            if count == 0 {
                Err(ApplicationError::NotFound(format!(
                    "Resume with id {} not found",
                    resume_id
                )))
            } else {
                Ok(())
            }
        }
        Err(err) => Err(app_err_from_diesel_err(err)),
    }
}
