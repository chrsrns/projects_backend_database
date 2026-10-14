use diesel::prelude::*;
use domain::models::{WorkExperience, WorkExperienceKeyPoint};
use infrastructure::establish_connection;

use crate::{
    error::ApplicationError,
    realtime::{ResumeChangePublisher, ResumeChangedAction, SectionType},
    resume::common::{app_err_from_diesel_err, first_or_404, require_owned_resume},
};

pub fn delete_work_experience(
    user_id_value: i32,
    work_id_value: i32,
    publisher: &dyn ResumeChangePublisher,
) -> Result<i32, ApplicationError> {
    use domain::schema::work_experiences;

    let existing: WorkExperience = first_or_404(
        "Work experience",
        work_id_value,
        work_experiences::table.find(work_id_value),
    )?;

    require_owned_resume(user_id_value, existing.resume_id)?;

    match diesel::delete(work_experiences::table.find(work_id_value))
        .execute(&mut establish_connection())
    {
        Ok(count) => {
            if count == 0 {
                Err(ApplicationError::NotFound(format!(
                    "Work experience with id {} not found",
                    work_id_value
                )))
            } else {
                publisher.resume_changed(
                    existing.resume_id,
                    ResumeChangedAction::Updated(SectionType::Experience),
                );
                Ok(existing.resume_id)
            }
        }
        Err(err) => Err(app_err_from_diesel_err(err)),
    }
}

pub fn delete_work_experience_key_point(
    user_id_value: i32,
    kp_id_value: i32,
    publisher: &dyn ResumeChangePublisher,
) -> Result<i32, ApplicationError> {
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

    match diesel::delete(work_experience_key_points::table.find(kp_id_value))
        .execute(&mut establish_connection())
    {
        Ok(count) => {
            if count == 0 {
                Err(ApplicationError::NotFound(format!(
                    "Work experience key point with id {} not found",
                    kp_id_value
                )))
            } else {
                publisher.resume_changed(
                    work.resume_id,
                    ResumeChangedAction::Updated(SectionType::Experience),
                );
                Ok(work.resume_id)
            }
        }
        Err(err) => Err(app_err_from_diesel_err(err)),
    }
}
