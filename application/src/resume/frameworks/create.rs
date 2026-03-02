use diesel::prelude::*;
use domain::models::{Framework, Language, NewFramework, NewFrameworkRequest, Resume};
use infrastructure::establish_connection;

use crate::error::ApplicationError;

pub fn create_framework(
    user_id_value: i32,
    resume_id_value: i32,
    language_id_value: i32,
    payload: NewFrameworkRequest,
) -> Result<Framework, ApplicationError> {
    use domain::schema::frameworks;
    use domain::schema::languages::dsl as languages_dsl;
    use domain::schema::resumes;

    let existing_resume: Resume = match resumes::table
        .find(resume_id_value)
        .first(&mut establish_connection())
    {
        Ok(r) => r,
        Err(diesel::result::Error::NotFound) => {
            return Err(ApplicationError::NotFound(format!(
                "Resume with id {} not found",
                resume_id_value
            )));
        }
        Err(err) => {
            return Err(ApplicationError::Internal(format!(
                "Database error - {}",
                err
            )));
        }
    };

    match existing_resume.created_by {
        Some(owner) if owner == user_id_value => {}
        Some(_) | None => {
            return Err(ApplicationError::Forbidden);
        }
    }

    let _language: Language = match languages_dsl::languages
        .filter(languages_dsl::id.eq(language_id_value))
        .filter(languages_dsl::resume_id.eq(resume_id_value))
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

    let new_framework = NewFramework {
        language_id: language_id_value,
        framework_name: payload.framework_name,
        display_order: payload.display_order,
    };

    match diesel::insert_into(frameworks::table)
        .values(&new_framework)
        .get_result::<Framework>(&mut establish_connection())
    {
        Ok(framework) => Ok(framework),
        Err(err) => Err(ApplicationError::Internal(format!(
            "Database error - {}",
            err
        ))),
    }
}
