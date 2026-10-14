use diesel::prelude::*;
use domain::models::{Language, UpdateLanguage};
use infrastructure::establish_connection;

use crate::{
    error::ApplicationError,
    realtime::{ResumeChangePublisher, ResumeChangedAction, SectionType},
    resume::common::{app_err_from_diesel_err, first_or_404, require_owned_resume},
};

pub fn update_language(
    user_id_value: i32,
    language_id_value: i32,
    payload: UpdateLanguage,
    publisher: &dyn ResumeChangePublisher,
) -> Result<Language, ApplicationError> {
    use domain::schema::languages;

    let existing: Language = first_or_404(
        "Language",
        language_id_value,
        languages::table.find(language_id_value),
    )?;

    require_owned_resume(user_id_value, existing.resume_id)?;

    match diesel::update(languages::table.find(language_id_value))
        .set(&payload)
        .get_result::<Language>(&mut establish_connection())
    {
        Ok(updated) => {
            publisher.resume_changed(
                updated.resume_id,
                ResumeChangedAction::Updated(SectionType::Languages),
            );
            Ok(updated)
        }
        Err(err) => Err(app_err_from_diesel_err(err)),
    }
}
