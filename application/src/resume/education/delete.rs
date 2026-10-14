use diesel::prelude::*;
use domain::models::{Education, EducationKeyPoint};
use infrastructure::establish_connection;

use crate::{
    error::ApplicationError,
    realtime::{ResumeChangePublisher, ResumeChangedAction, SectionType},
    resume::common::{app_err_from_diesel_err, first_or_404, require_owned_resume},
};

pub fn delete_education(
    user_id_value: i32,
    education_id_value: i32,
    publisher: &dyn ResumeChangePublisher,
) -> Result<i32, ApplicationError> {
    use domain::schema::education;

    let existing: Education = first_or_404(
        "Education",
        education_id_value,
        education::table.find(education_id_value),
    )?;

    require_owned_resume(user_id_value, existing.resume_id)?;

    match diesel::delete(education::table.find(education_id_value))
        .execute(&mut establish_connection())
    {
        Ok(count) => {
            if count == 0 {
                Err(ApplicationError::NotFound(format!(
                    "Education with id {} not found",
                    education_id_value
                )))
            } else {
                publisher.resume_changed(
                    existing.resume_id,
                    ResumeChangedAction::Updated(SectionType::Education),
                );
                Ok(existing.resume_id)
            }
        }
        Err(err) => Err(app_err_from_diesel_err(err)),
    }
}

pub fn delete_education_key_point(
    user_id_value: i32,
    key_point_id_value: i32,
    publisher: &dyn ResumeChangePublisher,
) -> Result<i32, ApplicationError> {
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

    match diesel::delete(education_key_points::table.find(key_point_id_value))
        .execute(&mut establish_connection())
    {
        Ok(count) => {
            if count == 0 {
                Err(ApplicationError::NotFound(format!(
                    "Education key point with id {} not found",
                    key_point_id_value
                )))
            } else {
                publisher.resume_changed(
                    edu.resume_id,
                    ResumeChangedAction::Updated(SectionType::Education),
                );
                Ok(edu.resume_id)
            }
        }
        Err(err) => Err(app_err_from_diesel_err(err)),
    }
}
