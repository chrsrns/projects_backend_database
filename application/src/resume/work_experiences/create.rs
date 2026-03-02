use diesel::prelude::*;
use domain::models::{
    NewWorkExperience, NewWorkExperienceKeyPoint, NewWorkExperienceKeyPointRequest,
    NewWorkExperienceRequest, Resume, WorkExperience, WorkExperienceKeyPoint,
};
use infrastructure::establish_connection;

use crate::error::ApplicationError;

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
    use domain::schema::resumes;
    use domain::schema::work_experiences;

    let resume: Resume = match resumes::table
        .find(resume_id_value)
        .first(&mut establish_connection())
    {
        Ok(r) => r,
        Err(diesel::result::Error::NotFound) => {
            return Err(ApplicationError::NotFound(format!(
                "Resume with id {} not found",
                resume_id_value
            )));
        }
        Err(err) => {
            return Err(ApplicationError::Internal(format!(
                "Database error - {}",
                err
            )));
        }
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
        Err(err) => Err(ApplicationError::Internal(format!(
            "Database error - {}",
            err
        ))),
    }
}

pub fn create_work_experience_key_point(
    user_id_value: i32,
    resume_id_value: i32,
    work_id_value: i32,
    payload: NewWorkExperienceKeyPointRequest,
) -> Result<WorkExperienceKeyPoint, ApplicationError> {
    use domain::schema::resumes;
    use domain::schema::work_experience_key_points;
    use domain::schema::work_experiences::dsl as work_dsl;

    let resume: Resume = match resumes::table
        .find(resume_id_value)
        .first(&mut establish_connection())
    {
        Ok(r) => r,
        Err(diesel::result::Error::NotFound) => {
            return Err(ApplicationError::NotFound(format!(
                "Resume with id {} not found",
                resume_id_value
            )));
        }
        Err(err) => {
            return Err(ApplicationError::Internal(format!(
                "Database error - {}",
                err
            )));
        }
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
        Err(diesel::result::Error::NotFound) => {
            return Err(ApplicationError::NotFound(
                "Work experience not found".to_string(),
            ));
        }
        Err(err) => {
            return Err(ApplicationError::Internal(format!(
                "Database error - {}",
                err
            )));
        }
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
        Err(err) => Err(ApplicationError::Internal(format!(
            "Database error - {}",
            err
        ))),
    }
}
