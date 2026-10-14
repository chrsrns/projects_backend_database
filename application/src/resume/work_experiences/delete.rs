use diesel::prelude::*;
use domain::models::{WorkExperience, WorkExperienceKeyPoint};

use crate::{
    error::ApplicationError,
    realtime::{ResumeChangePublisher, ResumeChangedAction, SectionType},
    resume::common::{first_or_404, require_owned_resume, run_published_delete},
};

pub fn delete_work_experience(
    user_id_value: i32,
    work_id_value: i32,
    publisher: &dyn ResumeChangePublisher,
) -> Result<(), ApplicationError> {
    use domain::schema::work_experiences;

    let existing: WorkExperience = first_or_404(
        "Work experience",
        work_id_value,
        work_experiences::table.find(work_id_value),
    )?;

    require_owned_resume(user_id_value, existing.resume_id)?;

    run_published_delete(
        "Work experience",
        work_id_value,
        existing.resume_id,
        ResumeChangedAction::Updated(SectionType::Experience),
        publisher,
        |conn| diesel::delete(work_experiences::table.find(work_id_value)).execute(conn),
    )
}

pub fn delete_work_experience_key_point(
    user_id_value: i32,
    kp_id_value: i32,
    publisher: &dyn ResumeChangePublisher,
) -> Result<(), ApplicationError> {
    use domain::schema::work_experience_key_points;
    use domain::schema::work_experiences;

    let existing: WorkExperienceKeyPoint = first_or_404(
        "Work experience key point",
        kp_id_value,
        work_experience_key_points::table.find(kp_id_value),
    )?;

    let work: WorkExperience = first_or_404(
        "Work experience",
        existing.work_experience_id,
        work_experiences::table.find(existing.work_experience_id),
    )?;

    require_owned_resume(user_id_value, work.resume_id)?;

    run_published_delete(
        "Work experience key point",
        kp_id_value,
        work.resume_id,
        ResumeChangedAction::Updated(SectionType::Experience),
        publisher,
        |conn| diesel::delete(work_experience_key_points::table.find(kp_id_value)).execute(conn),
    )
}
