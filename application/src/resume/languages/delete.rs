use diesel::prelude::*;
use domain::models::Language;
use infrastructure::establish_connection;

use crate::{
    error::ApplicationError,
    realtime::{ResumeChangePublisher, ResumeChangedAction, SectionType},
    resume::common::{app_err_from_diesel_err, first_or_404, require_owned_resume},
};

pub fn delete_language(
    user_id_value: i32,
    language_id_value: i32,
    publisher: &dyn ResumeChangePublisher,
) -> Result<i32, ApplicationError> {
    use domain::schema::languages;

    let existing: Language = first_or_404(
        "Language",
        language_id_value,
        languages::table.find(language_id_value),
    )?;

    require_owned_resume(user_id_value, existing.resume_id)?;

    match diesel::delete(languages::table.find(language_id_value))
        .execute(&mut establish_connection())
    {
        Ok(count) => {
            if count == 0 {
                Err(ApplicationError::NotFound(format!(
                    "Language with id {} not found",
                    language_id_value
                )))
            } else {
                publisher.resume_changed(
                    existing.resume_id,
                    ResumeChangedAction::Updated(SectionType::Languages),
                );
                Ok(existing.resume_id)
            }
        }
        Err(err) => Err(app_err_from_diesel_err(err)),
    }
}
