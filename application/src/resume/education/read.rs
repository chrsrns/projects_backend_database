use diesel::prelude::*;
use domain::models::{Education, EducationKeyPoint, Resume};
use infrastructure::establish_connection;

use crate::{error::ApplicationError, resume::common::app_err_from_diesel_err};

pub fn list_educations(
    resume_id_value: i32,
    user_id_value: Option<i32>,
) -> Result<Vec<Education>, ApplicationError> {
    use domain::schema::education::dsl as education_dsl;
    use domain::schema::resumes::dsl as resumes_dsl;

    let mut resume_query = resumes_dsl::resumes.into_boxed();
    resume_query = resume_query.filter(resumes_dsl::id.eq(resume_id_value));
    resume_query = match user_id_value {
        Some(uid) => resume_query.filter(
            resumes_dsl::is_public
                .eq(true)
                .or(resumes_dsl::created_by.eq(uid)),
        ),
        None => resume_query.filter(resumes_dsl::is_public.eq(true)),
    };

    let _resume: Resume = match resume_query.first(&mut establish_connection()) {
        Ok(r) => r,
        Err(err) => return Err(app_err_from_diesel_err(err)),
    };

    let mut items: Vec<Education> = match education_dsl::education
        .filter(education_dsl::resume_id.eq(resume_id_value))
        .load::<Education>(&mut establish_connection())
    {
        Ok(v) => v,
        Err(err) => return Err(app_err_from_diesel_err(err)),
    };

    items.sort_by_key(|e| (e.display_order.unwrap_or(0), e.id));

    Ok(items)
}

pub fn list_education_key_points(
    resume_id_value: i32,
    education_id_value: i32,
    user_id_value: Option<i32>,
) -> Result<Vec<EducationKeyPoint>, ApplicationError> {
    use domain::schema::education::dsl as education_dsl;
    use domain::schema::education_key_points::dsl as key_points_dsl;
    use domain::schema::resumes::dsl as resumes_dsl;

    let mut resume_query = resumes_dsl::resumes.into_boxed();
    resume_query = resume_query.filter(resumes_dsl::id.eq(resume_id_value));
    resume_query = match user_id_value {
        Some(uid) => resume_query.filter(
            resumes_dsl::is_public
                .eq(true)
                .or(resumes_dsl::created_by.eq(uid)),
        ),
        None => resume_query.filter(resumes_dsl::is_public.eq(true)),
    };

    let _resume: Resume = match resume_query.first(&mut establish_connection()) {
        Ok(r) => r,
        Err(err) => {
            return Err(app_err_from_diesel_err(err));
        }
    };

    let _education: Education = match education_dsl::education
        .filter(education_dsl::id.eq(education_id_value))
        .filter(education_dsl::resume_id.eq(resume_id_value))
        .first(&mut establish_connection())
    {
        Ok(e) => e,
        Err(err) => {
            return Err(app_err_from_diesel_err(err));
        }
    };

    let mut items: Vec<EducationKeyPoint> = match key_points_dsl::education_key_points
        .filter(key_points_dsl::education_id.eq(education_id_value))
        .load::<EducationKeyPoint>(&mut establish_connection())
    {
        Ok(v) => v,
        Err(err) => {
            return Err(app_err_from_diesel_err(err));
        }
    };

    items.sort_by_key(|kp| (kp.display_order.unwrap_or(0), kp.id));

    Ok(items)
}
