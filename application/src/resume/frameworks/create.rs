use diesel::prelude::*;
use domain::models::{Framework, Language, NewFramework, NewFrameworkRequest, Resume};
use infrastructure::establish_connection;

use crate::{
    error::ApplicationError,
    resume::common::{app_err_from_diesel_err, find_resume},
};

pub fn create_framework(
    user_id_value: i32,
    resume_id_value: i32,
    language_id_value: i32,
    payload: NewFrameworkRequest,
) -> Result<Framework, ApplicationError> {
    use domain::schema::frameworks;
    use domain::schema::languages::dsl as languages_dsl;

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

    let _language: Language = match languages_dsl::languages
        .filter(languages_dsl::id.eq(language_id_value))
        .filter(languages_dsl::resume_id.eq(resume_id_value))
        .first(&mut establish_connection())
    {
        Ok(l) => l,
        Err(err) => return Err(app_err_from_diesel_err(err)),
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
        Err(err) => Err(app_err_from_diesel_err(err)),
    }
}
