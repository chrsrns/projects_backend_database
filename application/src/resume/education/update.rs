use diesel::prelude::*;
use domain::models::{
    Education, EducationKeyPoint, UpdateEducation, UpdateEducationKeyPoint, UpdateEducationRequest,
};

use crate::{
    error::ApplicationError,
    realtime::{ResumeChangePublisher, ResumeChangedAction, SectionType},
    resume::common::{first_or_404, require_owned_resume, run_published},
};

pub fn update_education(
    user_id_value: i32,
    education_id_value: i32,
    request: UpdateEducationRequest,
    publisher: &dyn ResumeChangePublisher,
) -> Result<Education, ApplicationError> {
    use domain::schema::education;

    let existing: Education = first_or_404(
        "Education",
        education_id_value,
        education::table.find(education_id_value),
    )?;

    require_owned_resume(user_id_value, existing.resume_id)?;

    let new_start = request
        .start_date
        .map(|pd| pd.canonical_start_date())
        .unwrap_or(existing.start_date);
    let new_end = match request.end_date {
        None => existing.end_date,
        Some(None) => None,
        Some(Some(pd)) => Some(pd.canonical_end_date()),
    };

    if let Some(end) = new_end
        && new_start > end
    {
        return Err(ApplicationError::BadRequest(
            "start_date is after end_date".to_string(),
        ));
    }

    let payload = UpdateEducation {
        education_stage: request.education_stage,
        institution_name: request.institution_name,
        degree: request.degree,
        start_date: request
            .start_date
            .as_ref()
            .map(|pd| pd.canonical_start_date()),
        start_date_precision: request
            .start_date
            .as_ref()
            .map(|pd| pd.precision.to_string()),
        end_date: match request.end_date {
            None => None,
            Some(None) => Some(None),
            Some(Some(pd)) => Some(Some(pd.canonical_end_date())),
        },
        end_date_precision: match request.end_date {
            None => None,
            Some(None) => Some(None),
            Some(Some(pd)) => Some(Some(pd.precision.to_string())),
        },
        description: request.description,
        display_order: request.display_order,
        active: request.active,
    };

    run_published(
        "Education",
        education_id_value,
        existing.resume_id,
        ResumeChangedAction::Updated(SectionType::Education),
        publisher,
        |conn| {
            diesel::update(education::table.find(education_id_value))
                .set(&payload)
                .get_result::<Education>(conn)
        },
    )
}

pub fn update_education_key_point(
    user_id_value: i32,
    key_point_id_value: i32,
    payload: UpdateEducationKeyPoint,
    publisher: &dyn ResumeChangePublisher,
) -> Result<EducationKeyPoint, ApplicationError> {
    use domain::schema::education;
    use domain::schema::education_key_points;

    let existing: EducationKeyPoint = first_or_404(
        "Education key point",
        key_point_id_value,
        education_key_points::table.find(key_point_id_value),
    )?;

    let edu: Education = first_or_404(
        "Education",
        existing.education_id,
        education::table.find(existing.education_id),
    )?;

    require_owned_resume(user_id_value, edu.resume_id)?;

    run_published(
        "Education key point",
        key_point_id_value,
        edu.resume_id,
        ResumeChangedAction::Updated(SectionType::Education),
        publisher,
        |conn| {
            diesel::update(education_key_points::table.find(key_point_id_value))
                .set(&payload)
                .get_result::<EducationKeyPoint>(conn)
        },
    )
}
