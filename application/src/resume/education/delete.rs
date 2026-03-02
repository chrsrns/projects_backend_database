use diesel::prelude::*;
use domain::models::{Education, EducationKeyPoint, Resume};
use infrastructure::establish_connection;

use crate::error::ApplicationError;

pub fn delete_education(
    user_id_value: i32,
    education_id_value: i32,
) -> Result<i32, ApplicationError> {
    use domain::schema::education;
    use domain::schema::resumes;

    let existing: Education = match education::table
        .find(education_id_value)
        .first(&mut establish_connection())
    {
        Ok(v) => v,
        Err(diesel::result::Error::NotFound) => {
            return Err(ApplicationError::NotFound(format!(
                "Education with id {} not found",
                education_id_value
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
                Ok(existing.resume_id)
            }
        }
        Err(err) => Err(ApplicationError::Internal(format!(
            "Database error - {}",
            err
        ))),
    }
}

pub fn delete_education_key_point(
    user_id_value: i32,
    key_point_id_value: i32,
) -> Result<i32, ApplicationError> {
    use domain::schema::education;
    use domain::schema::education_key_points;
    use domain::schema::resumes;

    let existing: EducationKeyPoint = match education_key_points::table
        .find(key_point_id_value)
        .first(&mut establish_connection())
    {
        Ok(v) => v,
        Err(diesel::result::Error::NotFound) => {
            return Err(ApplicationError::NotFound(format!(
                "Education key point with id {} not found",
                key_point_id_value
            )));
        }
        Err(err) => {
            return Err(ApplicationError::Internal(format!(
                "Database error - {}",
                err
            )));
        }
    };

    let edu: Education = match education::table
        .find(existing.education_id)
        .first(&mut establish_connection())
    {
        Ok(e) => e,
        Err(diesel::result::Error::NotFound) => {
            return Err(ApplicationError::NotFound(
                "Education not found".to_string(),
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
        .find(edu.resume_id)
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
                Ok(edu.resume_id)
            }
        }
        Err(err) => Err(ApplicationError::Internal(format!(
            "Database error - {}",
            err
        ))),
    }
}
