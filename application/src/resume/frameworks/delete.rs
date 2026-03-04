use diesel::prelude::*;
use domain::models::{Framework, Language, Resume};
use infrastructure::establish_connection;

use crate::{
    error::ApplicationError,
    resume::common::{app_err_from_diesel_err, find_resume},
};

pub fn delete_framework(
    user_id_value: i32,
    framework_id_value: i32,
) -> Result<i32, ApplicationError> {
    use domain::schema::frameworks;
    use domain::schema::languages;

    let existing: Framework = match frameworks::table
        .find(framework_id_value)
        .first(&mut establish_connection())
    {
        Ok(r) => r,
        Err(err) => return Err(app_err_from_diesel_err(err)),
    };

    let language: Language = match languages::table
        .find(existing.language_id)
        .first(&mut establish_connection())
    {
        Ok(l) => l,
        Err(err) => return Err(app_err_from_diesel_err(err)),
    };

    let resume: Resume = match find_resume(language.resume_id) {
        Ok(r) => r,
        Err(err) => return Err(err),
    };

    match resume.created_by {
        Some(owner) if owner == user_id_value => {}
        Some(_) | None => {
            return Err(ApplicationError::Forbidden);
        }
    }

    match diesel::delete(frameworks::table.find(framework_id_value))
        .execute(&mut establish_connection())
    {
        Ok(count) => {
            if count == 0 {
                Err(ApplicationError::NotFound(format!(
                    "Framework with id {} not found",
                    framework_id_value
                )))
            } else {
                Ok(language.resume_id)
            }
        }
        Err(err) => Err(app_err_from_diesel_err(err)),
    }
}
