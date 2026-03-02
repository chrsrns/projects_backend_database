use diesel::prelude::*;
use domain::models::{Resume, Skill};
use infrastructure::establish_connection;

use crate::error::ApplicationError;

pub fn delete_skill(user_id_value: i32, skill_id_value: i32) -> Result<i32, ApplicationError> {
    use domain::schema::resumes;
    use domain::schema::skills;

    let existing: Skill = match skills::table
        .find(skill_id_value)
        .first(&mut establish_connection())
    {
        Ok(r) => r,
        Err(diesel::result::Error::NotFound) => {
            return Err(ApplicationError::NotFound(format!(
                "Skill with id {} not found",
                skill_id_value
            )));
        }
        Err(err) => {
            return Err(ApplicationError::Internal(format!(
                "Database error - {}",
                err
            )));
        }
    };

    let resume: Resume = match resumes::table
        .find(existing.resume_id)
        .first(&mut establish_connection())
    {
        Ok(r) => r,
        Err(diesel::result::Error::NotFound) => {
            return Err(ApplicationError::NotFound("Resume not found".to_string()));
        }
        Err(err) => {
            return Err(ApplicationError::Internal(format!(
                "Database error - {}",
                err
            )));
        }
    };

    match resume.created_by {
        Some(owner) if owner == user_id_value => {}
        Some(_) | None => {
            return Err(ApplicationError::Forbidden);
        }
    }

    match diesel::delete(skills::table.find(skill_id_value)).execute(&mut establish_connection()) {
        Ok(count) => {
            if count == 0 {
                Err(ApplicationError::NotFound(format!(
                    "Skill with id {} not found",
                    skill_id_value
                )))
            } else {
                Ok(existing.resume_id)
            }
        }
        Err(err) => Err(ApplicationError::Internal(format!(
            "Database error - {}",
            err
        ))),
    }
}
