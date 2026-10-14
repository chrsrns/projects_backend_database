use diesel::prelude::*;
use domain::models::{Education, EducationKeyPoint};

use crate::{
    error::ApplicationError,
    realtime::{ResumeChangePublisher, ResumeChangedAction, SectionType},
    resume::common::{first_or_404, require_owned_resume, run_published_delete},
};

pub fn delete_education(
    user_id_value: i32,
    education_id_value: i32,
    publisher: &dyn ResumeChangePublisher,
) -> Result<(), ApplicationError> {
    use domain::schema::education;

    let existing: Education = first_or_404(
        "Education",
        education_id_value,
        education::table.find(education_id_value),
    )?;

    require_owned_resume(user_id_value, existing.resume_id)?;

    run_published_delete(
        "Education",
        education_id_value,
        existing.resume_id,
        ResumeChangedAction::Updated(SectionType::Education),
        publisher,
        |conn| diesel::delete(education::table.find(education_id_value)).execute(conn),
    )
}

pub fn delete_education_key_point(
    user_id_value: i32,
    key_point_id_value: i32,
    publisher: &dyn ResumeChangePublisher,
) -> Result<(), ApplicationError> {
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

    run_published_delete(
        "Education key point",
        key_point_id_value,
        edu.resume_id,
        ResumeChangedAction::Updated(SectionType::Education),
        publisher,
        |conn| diesel::delete(education_key_points::table.find(key_point_id_value)).execute(conn),
    )
}
