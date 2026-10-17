use diesel::prelude::*;
use domain::models::{Language, NewLanguage, NewLanguageRequest};

use uuid::Uuid;

use crate::{
    error::ApplicationError,
    realtime::{ResumeChangePublisher, ResumeChangedAction, SectionType},
    resume::common::{require_owned_resume, run_published},
};

pub fn create_language(
    user_id_value: i32,
    resume_id_value: Uuid,
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

    run_published(
        "Language",
        0,
        resume_id_value,
        ResumeChangedAction::Updated(SectionType::Languages),
        publisher,
        |conn| {
            diesel::insert_into(languages::table)
                .values(&new_language)
                .get_result::<Language>(conn)
        },
    )
}
