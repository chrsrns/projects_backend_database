use diesel::prelude::*;
use domain::models::{
    Education, EducationKeyPoint, Resume, UpdateEducation, UpdateEducationKeyPoint,
};
use infrastructure::establish_connection;

use crate::{
    error::ApplicationError,
    resume::common::{app_err_from_diesel_err, find_resume},
};

pub fn update_education(
    user_id_value: i32,
    education_id_value: i32,
    payload: UpdateEducation,
) -> Result<Education, ApplicationError> {
    use domain::schema::education;

    let existing: Education = match education::table
        .find(education_id_value)
        .first(&mut establish_connection())
    {
        Ok(v) => v,
        Err(err) => {
            return Err(app_err_from_diesel_err(err));
        }
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

    match diesel::update(education::table.find(education_id_value))
        .set(&payload)
        .get_result::<Education>(&mut establish_connection())
    {
        Ok(updated) => Ok(updated),
        Err(err) => Err(app_err_from_diesel_err(err)),
    }
}

pub fn update_education_key_point(
    user_id_value: i32,
    key_point_id_value: i32,
    payload: UpdateEducationKeyPoint,
) -> Result<(EducationKeyPoint, i32), ApplicationError> {
    use domain::schema::education;
    use domain::schema::education_key_points;

    let existing: EducationKeyPoint = match education_key_points::table
        .find(key_point_id_value)
        .first(&mut establish_connection())
    {
        Ok(v) => v,
        Err(err) => {
            return Err(app_err_from_diesel_err(err));
        }
    };

    let edu: Education = match education::table
        .find(existing.education_id)
        .first(&mut establish_connection())
    {
        Ok(e) => e,
        Err(err) => {
            return Err(app_err_from_diesel_err(err));
        }
    };

    let resume: Resume = match find_resume(edu.resume_id) {
        Ok(r) => r,
        Err(err) => return Err(err),
    };

    match resume.created_by {
        Some(owner) if owner == user_id_value => {}
        Some(_) | None => {
            return Err(ApplicationError::Forbidden);
        }
    }

    match diesel::update(education_key_points::table.find(key_point_id_value))
        .set(&payload)
        .get_result::<EducationKeyPoint>(&mut establish_connection())
    {
        Ok(updated) => Ok((updated, edu.resume_id)),
        Err(err) => Err(app_err_from_diesel_err(err)),
    }
}
