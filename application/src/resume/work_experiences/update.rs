use diesel::prelude::*;
use domain::models::{
    Resume, UpdateWorkExperience, UpdateWorkExperienceKeyPoint, WorkExperience,
    WorkExperienceKeyPoint,
};
use infrastructure::establish_connection;

use crate::{
    error::ApplicationError,
    resume::common::{app_err_from_diesel_err, find_resume},
};

fn normalize_optional_string_change(value: Option<Option<String>>) -> Option<Option<String>> {
    value.map(|inner| {
        inner.and_then(|v| {
            let trimmed = v.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        })
    })
}

pub fn update_work_experience(
    user_id_value: i32,
    work_id_value: i32,
    payload: UpdateWorkExperience,
) -> Result<WorkExperience, ApplicationError> {
    use domain::schema::work_experiences;

    let existing: WorkExperience = match work_experiences::table
        .find(work_id_value)
        .first(&mut establish_connection())
    {
        Ok(v) => v,
        Err(err) => return Err(app_err_from_diesel_err(err)),
    };

    let resume: Resume = match find_resume(existing.resume_id) {
        Ok(r) => r,
        Err(err) => return Err(err),
    };

    match resume.created_by {
        Some(owner) if owner == user_id_value => {}
        Some(_) | None => {
            return Err(ApplicationError::Forbidden);
        }
    }

    let payload = UpdateWorkExperience {
        job_title: payload.job_title.map(|v| v.trim().to_string()),
        company_name: normalize_optional_string_change(payload.company_name),
        start_date: payload.start_date,
        end_date: payload.end_date,
        description: normalize_optional_string_change(payload.description),
        display_order: payload.display_order,
    };
    match diesel::update(work_experiences::table.find(work_id_value))
        .set(&payload)
        .get_result::<WorkExperience>(&mut establish_connection())
    {
        Ok(updated) => Ok(updated),
        Err(err) => Err(app_err_from_diesel_err(err)),
    }
}

pub fn update_work_experience_key_point(
    user_id_value: i32,
    kp_id_value: i32,
    payload: UpdateWorkExperienceKeyPoint,
) -> Result<(WorkExperienceKeyPoint, i32), ApplicationError> {
    use domain::schema::work_experience_key_points;
    use domain::schema::work_experiences;

    let existing: WorkExperienceKeyPoint = match work_experience_key_points::table
        .find(kp_id_value)
        .first(&mut establish_connection())
    {
        Ok(v) => v,
        Err(err) => return Err(app_err_from_diesel_err(err)),
    };

    let work: WorkExperience = match work_experiences::table
        .find(existing.work_experience_id)
        .first(&mut establish_connection())
    {
        Ok(w) => w,
        Err(err) => return Err(app_err_from_diesel_err(err)),
    };

    let resume: Resume = match find_resume(work.resume_id) {
        Ok(r) => r,
        Err(eval) => return Err(eval),
    };

    match resume.created_by {
        Some(owner) if owner == user_id_value => {}
        Some(_) | None => {
            return Err(ApplicationError::Forbidden);
        }
    }

    match diesel::update(work_experience_key_points::table.find(kp_id_value))
        .set(&payload)
        .get_result::<WorkExperienceKeyPoint>(&mut establish_connection())
    {
        Ok(updated) => Ok((updated, work.resume_id)),
        Err(err) => Err(app_err_from_diesel_err(err)),
    }
}
