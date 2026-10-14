use diesel::prelude::*;
use domain::models::{Framework, Language, NewFramework, NewFrameworkRequest};

use crate::{
    error::ApplicationError,
    realtime::{ResumeChangePublisher, ResumeChangedAction, SectionType},
    resume::common::{first_or_404, require_owned_resume, run_published},
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

    require_owned_resume(user_id_value, resume_id_value)?;

    let _language: Language = first_or_404(
        "Language",
        language_id_value,
        languages_dsl::languages
            .filter(languages_dsl::id.eq(language_id_value))
            .filter(languages_dsl::resume_id.eq(resume_id_value)),
    )?;

    let new_framework = NewFramework {
        language_id: language_id_value,
        framework_name: payload.framework_name,
        display_order: payload.display_order,
    };

    run_published(
        "Framework",
        0,
        resume_id_value,
        ResumeChangedAction::Updated(SectionType::Frameworks),
        publisher,
        |conn| {
            diesel::insert_into(frameworks::table)
                .values(&new_framework)
                .get_result::<Framework>(conn)
        },
    )
}
