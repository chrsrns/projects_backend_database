use diesel::prelude::*;
use domain::models::{Framework, Language, Resume, UpdateFramework};
use infrastructure::establish_connection;

use crate::{
    error::ApplicationError,
    resume::common::{app_err_from_diesel_err, find_resume},
};

pub fn update_framework(
    user_id_value: i32,
    framework_id_value: i32,
    payload: UpdateFramework,
) -> Result<(Framework, i32), ApplicationError> {
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

    match diesel::update(frameworks::table.find(framework_id_value))
        .set(&payload)
        .get_result::<Framework>(&mut establish_connection())
    {
        Ok(updated) => Ok((updated, language.resume_id)),
        Err(err) => Err(app_err_from_diesel_err(err)),
    }
}
