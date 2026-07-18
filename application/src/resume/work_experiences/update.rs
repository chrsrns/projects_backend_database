use diesel::prelude::*;
use domain::models::{
    Resume, UpdateWorkExperience, UpdateWorkExperienceKeyPoint, UpdateWorkExperienceRequest,
    WorkExperience, WorkExperienceKeyPoint,
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
    request: UpdateWorkExperienceRequest,
) -> Result<WorkExperience, ApplicationError> {
    use domain::schema::work_experiences;

    let existing: WorkExperience = match work_experiences::table
        .find(work_id_value)
        .first(&mut establish_connection())
    {
        Ok(v) => v,
        Err(err) => return Err(app_err_from_diesel_err(err)),
    };

    let resume: Resume = find_resume(existing.resume_id)?;

    match resume.created_by {
        Some(owner) if owner == user_id_value => {}
        Some(_) | None => {
            return Err(ApplicationError::Forbidden);
        }
    }

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

    let payload = UpdateWorkExperience {
        job_title: request.job_title.map(|v| v.trim().to_string()),
        company_name: normalize_optional_string_change(request.company_name),
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
        description: normalize_optional_string_change(request.description),
        display_order: request.display_order,
        active: request.active,
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

    let resume: Resume = find_resume(work.resume_id)?;

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
