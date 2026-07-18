use diesel::prelude::*;
use domain::models::{
    Education, EducationKeyPoint, Resume, UpdateEducation, UpdateEducationKeyPoint,
    UpdateEducationRequest,
};
use infrastructure::establish_connection;

use crate::{
    error::ApplicationError,
    resume::common::{app_err_from_diesel_err, find_resume},
};

pub fn update_education(
    user_id_value: i32,
    education_id_value: i32,
    request: UpdateEducationRequest,
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

    let payload = UpdateEducation {
        education_stage: request.education_stage,
        institution_name: request.institution_name,
        degree: request.degree,
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
        description: request.description,
        display_order: request.display_order,
        active: request.active,
    };

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

    let resume: Resume = find_resume(edu.resume_id)?;

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
