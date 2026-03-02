use diesel::prelude::*;
use domain::models::{Language, Resume, UpdateLanguage};
use infrastructure::establish_connection;

use crate::error::ApplicationError;

pub fn update_language(
    user_id_value: i32,
    language_id_value: i32,
    payload: UpdateLanguage,
) -> Result<Language, ApplicationError> {
    use domain::schema::languages;
    use domain::schema::resumes;

    let existing: Language = match languages::table
        .find(language_id_value)
        .first(&mut establish_connection())
    {
        Ok(r) => r,
        Err(diesel::result::Error::NotFound) => {
            return Err(ApplicationError::NotFound(format!(
                "Language with id {} not found",
                language_id_value
            )));
        }
        Err(err) => return Err(ApplicationError::Internal(format!("Database error - {}", err))),
    };

    let resume: Resume = match resumes::table
        .find(existing.resume_id)
        .first(&mut establish_connection())
    {
        Ok(r) => r,
        Err(diesel::result::Error::NotFound) => {
            return Err(ApplicationError::NotFound("Resume not found".to_string()));
        }
        Err(err) => return Err(ApplicationError::Internal(format!("Database error - {}", err))),
    };

    match resume.created_by {
        Some(owner) if owner == user_id_value => {}
        Some(_) | None => {
            return Err(ApplicationError::Forbidden);
        }
    }

    match diesel::update(languages::table.find(language_id_value))
        .set(&payload)
        .get_result::<Language>(&mut establish_connection())
    {
        Ok(updated) => Ok(updated),
        Err(diesel::result::Error::NotFound) => {
            Err(ApplicationError::NotFound(format!(
                "Language with id {} not found",
                language_id_value
            )))
        }
        Err(err) => Err(ApplicationError::Internal(format!("Database error - {}", err))),
    }
}
