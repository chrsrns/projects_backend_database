use diesel::prelude::*;
use domain::models::{Language, UpdateLanguage};

use crate::{
    error::ApplicationError,
    realtime::{ResumeChangePublisher, ResumeChangedAction, SectionType},
    resume::common::{first_or_404, require_owned_resume, run_published},
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

    run_published(
        "Language",
        language_id_value,
        existing.resume_id,
        ResumeChangedAction::Updated(SectionType::Languages),
        publisher,
        |conn| {
            diesel::update(languages::table.find(language_id_value))
                .set(&payload)
                .get_result::<Language>(conn)
        },
    )
}
