use diesel::prelude::*;
use domain::models::{Framework, Language, Resume, UpdateFramework};
use infrastructure::establish_connection;

use crate::error::ApplicationError;

pub fn update_framework(
    user_id_value: i32,
    framework_id_value: i32,
    payload: UpdateFramework,
) -> Result<Framework, ApplicationError> {
    use domain::schema::frameworks;
    use domain::schema::languages;
    use domain::schema::resumes;

    let existing: Framework = match frameworks::table
        .find(framework_id_value)
        .first(&mut establish_connection())
    {
        Ok(r) => r,
        Err(diesel::result::Error::NotFound) => {
            return Err(ApplicationError::NotFound(format!(
                "Framework with id {} not found",
                framework_id_value
            )));
        }
        Err(err) => {
            return Err(ApplicationError::Internal(format!(
                "Database error - {}",
                err
            )));
        }
    };

    let language: Language = match languages::table
        .find(existing.language_id)
        .first(&mut establish_connection())
    {
        Ok(l) => l,
        Err(diesel::result::Error::NotFound) => {
            return Err(ApplicationError::NotFound("Language not found".to_string()));
        }
        Err(err) => {
            return Err(ApplicationError::Internal(format!(
                "Database error - {}",
                err
            )));
        }
    };

    let resume: Resume = match resumes::table
        .find(language.resume_id)
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

    match diesel::update(frameworks::table.find(framework_id_value))
        .set(&payload)
        .get_result::<Framework>(&mut establish_connection())
    {
        Ok(updated) => Ok(updated),
        Err(diesel::result::Error::NotFound) => Err(ApplicationError::NotFound(format!(
            "Framework with id {} not found",
            framework_id_value
        ))),
        Err(err) => Err(ApplicationError::Internal(format!(
            "Database error - {}",
            err
        ))),
    }
}
