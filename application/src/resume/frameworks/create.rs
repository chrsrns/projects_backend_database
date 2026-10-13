use diesel::prelude::*;
use domain::models::{Framework, Language, NewFramework, NewFrameworkRequest, Resume};
use infrastructure::establish_connection;

use crate::{
    error::ApplicationError,
    realtime::{ResumeChangePublisher, ResumeChangedAction, SectionType},
    resume::common::{app_err_from_diesel_err, find_resume, require_owner},
};

pub fn create_framework(
    user_id_value: i32,
    resume_id_value: i32,
    language_id_value: i32,
    payload: NewFrameworkRequest,
    publisher: &dyn ResumeChangePublisher,
) -> Result<Framework, ApplicationError> {
    use domain::schema::frameworks;
    use domain::schema::languages::dsl as languages_dsl;

    let existing_resume: Resume = find_resume(resume_id_value)?;

    require_owner(&existing_resume, user_id_value)?;

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
        Ok(framework) => {
            publisher.resume_changed(
                resume_id_value,
                ResumeChangedAction::Updated(SectionType::Frameworks),
            );
            Ok(framework)
        }
        Err(err) => Err(app_err_from_diesel_err(err)),
    }
}
