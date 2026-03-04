use diesel::prelude::*;
use domain::models::{
    NewWorkExperience, NewWorkExperienceKeyPoint, NewWorkExperienceKeyPointRequest,
    NewWorkExperienceRequest, Resume, WorkExperience, WorkExperienceKeyPoint,
};
use infrastructure::establish_connection;

use crate::{
    error::ApplicationError,
    resume::common::{app_err_from_diesel_err, find_resume},
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
) -> Result<WorkExperience, ApplicationError> {
    use domain::schema::work_experiences;

    let resume: Resume = match find_resume(resume_id_value) {
        Ok(r) => r,
        Err(err) => return Err(err),
    };

    match resume.created_by {
        Some(owner) if owner == user_id_value => {}
        Some(_) | None => {
            return Err(ApplicationError::Forbidden);
        }
    }

    let company_name = normalize_optional_text(payload.company_name);
    let description = normalize_optional_text(payload.description);
    let new_item = NewWorkExperience {
        resume_id: resume_id_value,
        job_title: payload.job_title,
        company_name,
        start_date: payload.start_date,
        end_date: payload.end_date,
        description,
        display_order: payload.display_order,
    };

    match diesel::insert_into(work_experiences::table)
        .values(&new_item)
        .get_result::<WorkExperience>(&mut establish_connection())
    {
        Ok(item) => Ok(item),
        Err(err) => Err(app_err_from_diesel_err(err)),
    }
}

pub fn create_work_experience_key_point(
    user_id_value: i32,
    resume_id_value: i32,
    work_id_value: i32,
    payload: NewWorkExperienceKeyPointRequest,
) -> Result<WorkExperienceKeyPoint, ApplicationError> {
    use domain::schema::work_experience_key_points;
    use domain::schema::work_experiences::dsl as work_dsl;

    let resume: Resume = match find_resume(resume_id_value) {
        Ok(r) => r,
        Err(err) => return Err(err),
    };

    match resume.created_by {
        Some(owner) if owner == user_id_value => {}
        Some(_) | None => {
            return Err(ApplicationError::Forbidden);
        }
    }

    let _work: WorkExperience = match work_dsl::work_experiences
        .filter(work_dsl::id.eq(work_id_value))
        .filter(work_dsl::resume_id.eq(resume_id_value))
        .first(&mut establish_connection())
    {
        Ok(w) => w,
        Err(err) => return Err(app_err_from_diesel_err(err)),
    };

    let new_kp = NewWorkExperienceKeyPoint {
        work_experience_id: work_id_value,
        key_point: payload.key_point,
        display_order: payload.display_order,
    };

    match diesel::insert_into(work_experience_key_points::table)
        .values(&new_kp)
        .get_result::<WorkExperienceKeyPoint>(&mut establish_connection())
    {
        Ok(item) => Ok(item),
        Err(err) => Err(app_err_from_diesel_err(err)),
    }
}
