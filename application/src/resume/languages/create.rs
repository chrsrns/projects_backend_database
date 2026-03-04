use diesel::prelude::*;
use domain::models::{Language, NewLanguage, NewLanguageRequest, Resume};
use infrastructure::establish_connection;

use crate::{
    error::ApplicationError,
    resume::common::{app_err_from_diesel_err, find_resume},
};

pub fn create_language(
    user_id_value: i32,
    resume_id_value: i32,
    payload: NewLanguageRequest,
) -> Result<Language, ApplicationError> {
    use domain::schema::languages;

    let existing_resume: Resume = match find_resume(resume_id_value) {
        Ok(r) => r,
        Err(err) => return Err(err),
    };

    match existing_resume.created_by {
        Some(owner) if owner == user_id_value => {}
        Some(_) | None => {
            return Err(ApplicationError::Forbidden);
        }
    }

    let new_language = NewLanguage {
        resume_id: resume_id_value,
        language_name: payload.language_name,
        display_order: payload.display_order,
    };

    match diesel::insert_into(languages::table)
        .values(&new_language)
        .get_result::<Language>(&mut establish_connection())
    {
        Ok(language) => Ok(language),
        Err(err) => Err(app_err_from_diesel_err(err)),
    }
}
