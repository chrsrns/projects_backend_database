use diesel::prelude::*;
use domain::models::{Language, NewLanguage, NewLanguageRequest};
use infrastructure::establish_connection;

use crate::{
    error::ApplicationError,
    realtime::{ResumeChangePublisher, ResumeChangedAction, SectionType},
    resume::common::{app_err_from_diesel_err, require_owned_resume},
};

pub fn create_language(
    user_id_value: i32,
    resume_id_value: i32,
    payload: NewLanguageRequest,
    publisher: &dyn ResumeChangePublisher,
) -> Result<Language, ApplicationError> {
    use domain::schema::languages;

    require_owned_resume(user_id_value, resume_id_value)?;

    let new_language = NewLanguage {
        resume_id: resume_id_value,
        language_name: payload.language_name,
        display_order: payload.display_order,
    };

    match diesel::insert_into(languages::table)
        .values(&new_language)
        .get_result::<Language>(&mut establish_connection())
    {
        Ok(language) => {
            publisher.resume_changed(
                resume_id_value,
                ResumeChangedAction::Updated(SectionType::Languages),
            );
            Ok(language)
        }
        Err(err) => Err(app_err_from_diesel_err(err)),
    }
}
