use diesel::prelude::*;
use domain::models::{Framework, Language, Resume};
use infrastructure::establish_connection;

use crate::{
    error::ApplicationError,
    realtime::{ResumeChangePublisher, ResumeChangedAction, SectionType},
    resume::common::{app_err_from_diesel_err, find_resume, require_owner},
};

pub fn delete_framework(
    user_id_value: i32,
    framework_id_value: i32,
    publisher: &dyn ResumeChangePublisher,
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

    let resume: Resume = find_resume(language.resume_id)?;

    require_owner(&resume, user_id_value)?;

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
                publisher.resume_changed(
                    language.resume_id,
                    ResumeChangedAction::Updated(SectionType::Frameworks),
                );
                Ok(language.resume_id)
            }
        }
        Err(err) => Err(app_err_from_diesel_err(err)),
    }
}
