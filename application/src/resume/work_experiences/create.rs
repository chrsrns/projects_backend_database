use diesel::prelude::*;
use domain::models::{
    NewWorkExperience, NewWorkExperienceKeyPoint, NewWorkExperienceKeyPointRequest,
    NewWorkExperienceRequest, WorkExperience, WorkExperienceKeyPoint,
};

use crate::{
    error::ApplicationError,
    realtime::{ResumeChangePublisher, ResumeChangedAction, SectionType},
    resume::common::{first_or_404, require_owned_resume, run_published},
};

fn normalize_optional_text(value: Option<String>) -> Option<String> {
    value.and_then(|v| {
        let trimmed = v.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

pub fn create_work_experience(
    user_id_value: i32,
    resume_id_value: i32,
    payload: NewWorkExperienceRequest,
    publisher: &dyn ResumeChangePublisher,
) -> Result<WorkExperience, ApplicationError> {
    use domain::schema::work_experiences;

    require_owned_resume(user_id_value, resume_id_value)?;

    if let Some(ref end) = payload.end_date
        && payload.start_date.canonical_start_date() > end.canonical_end_date()
    {
        return Err(ApplicationError::BadRequest(format!(
            "start_date {} is after end_date {}",
            payload.start_date.to_iso_string(),
            end.to_iso_string()
        )));
    }

    let company_name = normalize_optional_text(payload.company_name);
    let description = normalize_optional_text(payload.description);
    let end_pair = payload
        .end_date
        .map(|end| (end.canonical_end_date(), end.precision.to_string()));
    let new_item = NewWorkExperience {
        resume_id: resume_id_value,
        job_title: payload.job_title,
        company_name,
        start_date: payload.start_date.canonical_start_date(),
        start_date_precision: payload.start_date.precision.to_string(),
        end_date: end_pair.as_ref().map(|(d, _)| *d),
        end_date_precision: end_pair.map(|(_, p)| p),
        description,
        display_order: payload.display_order,
        active: true,
    };

    run_published(
        "Work experience",
        0,
        resume_id_value,
        ResumeChangedAction::Updated(SectionType::Experience),
        publisher,
        |conn| {
            diesel::insert_into(work_experiences::table)
                .values(&new_item)
                .get_result::<WorkExperience>(conn)
        },
    )
}

pub fn create_work_experience_key_point(
    user_id_value: i32,
    resume_id_value: i32,
    work_id_value: i32,
    payload: NewWorkExperienceKeyPointRequest,
    publisher: &dyn ResumeChangePublisher,
) -> Result<WorkExperienceKeyPoint, ApplicationError> {
    use domain::schema::work_experience_key_points;
    use domain::schema::work_experiences::dsl as work_dsl;

    require_owned_resume(user_id_value, resume_id_value)?;

    let _work: WorkExperience = first_or_404(
        "Work experience",
        work_id_value,
        work_dsl::work_experiences
            .filter(work_dsl::id.eq(work_id_value))
            .filter(work_dsl::resume_id.eq(resume_id_value)),
    )?;

    let new_kp = NewWorkExperienceKeyPoint {
        work_experience_id: work_id_value,
        key_point: payload.key_point,
        display_order: payload.display_order,
        active: true,
    };

    run_published(
        "Work experience key point",
        0,
        resume_id_value,
        ResumeChangedAction::Updated(SectionType::Experience),
        publisher,
        |conn| {
            diesel::insert_into(work_experience_key_points::table)
                .values(&new_kp)
                .get_result::<WorkExperienceKeyPoint>(conn)
        },
    )
}
