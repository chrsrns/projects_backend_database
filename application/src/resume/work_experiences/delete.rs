use diesel::prelude::*;
use domain::models::{Resume, WorkExperience, WorkExperienceKeyPoint};
use infrastructure::establish_connection;

use crate::error::ApplicationError;

pub fn delete_work_experience(
    user_id_value: i32,
    work_id_value: i32,
) -> Result<i32, ApplicationError> {
    use domain::schema::resumes;
    use domain::schema::work_experiences;

    let existing: WorkExperience = match work_experiences::table
        .find(work_id_value)
        .first(&mut establish_connection())
    {
        Ok(v) => v,
        Err(diesel::result::Error::NotFound) => {
            return Err(ApplicationError::NotFound(format!(
                "Work experience with id {} not found",
                work_id_value
            )));
        }
        Err(err) => {
            return Err(ApplicationError::Internal(format!(
                "Database error - {}",
                err
            )));
        }
    };

    let resume: Resume = match resumes::table
        .find(existing.resume_id)
        .first(&mut establish_connection())
    {
        Ok(r) => r,
        Err(diesel::result::Error::NotFound) => {
            return Err(ApplicationError::NotFound("Resume not found".to_string()));
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
                Ok(existing.resume_id)
            }
        }
        Err(err) => Err(ApplicationError::Internal(format!(
            "Database error - {}",
            err
        ))),
    }
}

pub fn delete_work_experience_key_point(
    user_id_value: i32,
    kp_id_value: i32,
) -> Result<i32, ApplicationError> {
    use domain::schema::resumes;
    use domain::schema::work_experience_key_points;
    use domain::schema::work_experiences;

    let existing: WorkExperienceKeyPoint = match work_experience_key_points::table
        .find(kp_id_value)
        .first(&mut establish_connection())
    {
        Ok(v) => v,
        Err(diesel::result::Error::NotFound) => {
            return Err(ApplicationError::NotFound(format!(
                "Work experience key point with id {} not found",
                kp_id_value
            )));
        }
        Err(err) => {
            return Err(ApplicationError::Internal(format!(
                "Database error - {}",
                err
            )));
        }
    };

    let work: WorkExperience = match work_experiences::table
        .find(existing.work_experience_id)
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

    let resume: Resume = match resumes::table
        .find(work.resume_id)
        .first(&mut establish_connection())
    {
        Ok(r) => r,
        Err(diesel::result::Error::NotFound) => {
            return Err(ApplicationError::NotFound("Resume not found".to_string()));
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
                Ok(work.resume_id)
            }
        }
        Err(err) => Err(ApplicationError::Internal(format!(
            "Database error - {}",
            err
        ))),
    }
}
