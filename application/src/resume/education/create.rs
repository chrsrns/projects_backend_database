use diesel::prelude::*;
use domain::models::{
    Education, EducationKeyPoint, NewEducation, NewEducationKeyPoint, NewEducationKeyPointRequest,
    NewEducationRequest,
};
use uuid::Uuid;

use crate::{
    error::ApplicationError,
    realtime::{ResumeChangePublisher, ResumeChangedAction, SectionType},
    resume::common::{first_or_404, require_owned_resume, run_published},
};

pub fn create_education(
    user_id_value: i32,
    resume_id_value: Uuid,
    payload: NewEducationRequest,
    publisher: &dyn ResumeChangePublisher,
) -> Result<Education, ApplicationError> {
    use domain::schema::education;

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

    let end_pair = payload
        .end_date
        .map(|end| (end.canonical_end_date(), end.precision.to_string()));

    let new_education = NewEducation {
        resume_id: resume_id_value,
        education_stage: payload.education_stage,
        institution_name: payload.institution_name,
        degree: payload.degree,
        start_date: payload.start_date.canonical_start_date(),
        start_date_precision: payload.start_date.precision.to_string(),
        end_date: end_pair.as_ref().map(|(d, _)| *d),
        end_date_precision: end_pair.map(|(_, p)| p),
        description: payload.description,
        display_order: payload.display_order,
        active: true,
    };

    run_published(
        "Education",
        0,
        resume_id_value,
        ResumeChangedAction::Updated(SectionType::Education),
        publisher,
        |conn| {
            diesel::insert_into(education::table)
                .values(&new_education)
                .get_result::<Education>(conn)
        },
    )
}

pub fn create_education_key_point(
    user_id_value: i32,
    resume_id_value: Uuid,
    education_id_value: i32,
    payload: NewEducationKeyPointRequest,
    publisher: &dyn ResumeChangePublisher,
) -> Result<EducationKeyPoint, ApplicationError> {
    use domain::schema::education::dsl as education_dsl;
    use domain::schema::education_key_points;

    require_owned_resume(user_id_value, resume_id_value)?;

    let _education: Education = first_or_404(
        "Education",
        education_id_value,
        education_dsl::education
            .filter(education_dsl::id.eq(education_id_value))
            .filter(education_dsl::resume_id.eq(resume_id_value)),
    )?;

    let new_kp = NewEducationKeyPoint {
        education_id: education_id_value,
        key_point: payload.key_point,
        display_order: payload.display_order,
        active: true,
    };

    run_published(
        "Education key point",
        0,
        resume_id_value,
        ResumeChangedAction::Updated(SectionType::Education),
        publisher,
        |conn| {
            diesel::insert_into(education_key_points::table)
                .values(&new_kp)
                .get_result::<EducationKeyPoint>(conn)
        },
    )
}
