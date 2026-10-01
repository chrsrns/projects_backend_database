use diesel::prelude::*;
use domain::models::{
    Education, EducationKeyPoint, NewEducation, NewEducationKeyPoint, NewEducationKeyPointRequest,
    NewEducationRequest, Resume,
};
use infrastructure::establish_connection;

use crate::{
    error::ApplicationError,
    resume::common::{app_err_from_diesel_err, find_resume},
};

pub fn create_education(
    user_id_value: i32,
    resume_id_value: i32,
    payload: NewEducationRequest,
) -> Result<Education, ApplicationError> {
    use domain::schema::education;

    let resume: Resume = find_resume(resume_id_value)?;

    match resume.created_by {
        Some(owner) if owner == user_id_value => {}
        Some(_) | None => {
            return Err(ApplicationError::Forbidden);
        }
    }

    if let Some(ref end) = payload.end_date
        && payload.start_date.canonical_start_date() > end.canonical_end_date()
    {
        return Err(ApplicationError::BadRequest(format!(
            "start_date {} is after end_date {}",
            payload.start_date.to_iso_string(),
            end.to_iso_string()
        )));
    }

    let end_pair = payload
        .end_date
        .map(|end| (end.canonical_end_date(), end.precision.to_string()));

    let new_education = NewEducation {
        resume_id: resume_id_value,
        education_stage: payload.education_stage,
        institution_name: payload.institution_name,
        degree: payload.degree,
        start_date: payload.start_date.canonical_start_date(),
        start_date_precision: payload.start_date.precision.to_string(),
        end_date: end_pair.as_ref().map(|(d, _)| *d),
        end_date_precision: end_pair.map(|(_, p)| p),
        description: payload.description,
        display_order: payload.display_order,
        active: true,
    };

    match diesel::insert_into(education::table)
        .values(&new_education)
        .get_result::<Education>(&mut establish_connection())
    {
        Ok(item) => Ok(item),
        Err(err) => Err(app_err_from_diesel_err(err)),
    }
}

pub fn create_education_key_point(
    user_id_value: i32,
    resume_id_value: i32,
    education_id_value: i32,
    payload: NewEducationKeyPointRequest,
) -> Result<EducationKeyPoint, ApplicationError> {
    use domain::schema::education::dsl as education_dsl;
    use domain::schema::education_key_points;

    let resume: Resume = find_resume(resume_id_value)?;

    match resume.created_by {
        Some(owner) if owner == user_id_value => {}
        Some(_) | None => {
            return Err(ApplicationError::Forbidden);
        }
    }

    let _education: Education = match education_dsl::education
        .filter(education_dsl::id.eq(education_id_value))
        .filter(education_dsl::resume_id.eq(resume_id_value))
        .first(&mut establish_connection())
    {
        Ok(e) => e,
        Err(err) => return Err(app_err_from_diesel_err(err)),
    };

    let new_kp = NewEducationKeyPoint {
        education_id: education_id_value,
        key_point: payload.key_point,
        display_order: payload.display_order,
        active: true,
    };

    match diesel::insert_into(education_key_points::table)
        .values(&new_kp)
        .get_result::<EducationKeyPoint>(&mut establish_connection())
    {
        Ok(item) => Ok(item),
        Err(err) => Err(app_err_from_diesel_err(err)),
    }
}
