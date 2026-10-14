use diesel::prelude::*;
use domain::models::{Framework, Language, UpdateFramework};

use crate::{
    error::ApplicationError,
    realtime::{ResumeChangePublisher, ResumeChangedAction, SectionType},
    resume::common::{first_or_404, require_owned_resume, run_published},
};

pub fn update_framework(
    user_id_value: i32,
    framework_id_value: i32,
    payload: UpdateFramework,
    publisher: &dyn ResumeChangePublisher,
) -> Result<Framework, ApplicationError> {
    use domain::schema::frameworks;
    use domain::schema::languages;

    let existing: Framework = first_or_404(
        "Framework",
        framework_id_value,
        frameworks::table.find(framework_id_value),
    )?;

    let language: Language = first_or_404(
        "Language",
        existing.language_id,
        languages::table.find(existing.language_id),
    )?;

    require_owned_resume(user_id_value, language.resume_id)?;

    run_published(
        "Framework",
        framework_id_value,
        language.resume_id,
        ResumeChangedAction::Updated(SectionType::Frameworks),
        publisher,
        |conn| {
            diesel::update(frameworks::table.find(framework_id_value))
                .set(&payload)
                .get_result::<Framework>(conn)
        },
    )
}
