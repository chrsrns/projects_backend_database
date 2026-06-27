use diesel::prelude::*;
use domain::models::{Education, EducationKeyPoint};
use infrastructure::establish_connection;

use crate::{
    error::ApplicationError,
    resume::common::{app_err_from_diesel_err, find_accessible_resume, find_resume},
};

pub fn list_educations(
    resume_id_value: i32,
    user_id_value: Option<i32>,
) -> Result<Vec<Education>, ApplicationError> {
    use domain::schema::education::dsl as education_dsl;

    find_accessible_resume(resume_id_value, user_id_value)?;

    let is_owner = match user_id_value {
        Some(uid) => match find_resume(resume_id_value) {
            Ok(resume) => resume.created_by == Some(uid),
            Err(_) => false,
        },
        None => false,
    };

    let mut query = education_dsl::education.into_boxed();
    query = query.filter(education_dsl::resume_id.eq(resume_id_value));
    if !is_owner {
        query = query.filter(education_dsl::active.eq(true));
    }

    let mut items: Vec<Education> = match query.load::<Education>(&mut establish_connection()) {
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

    find_accessible_resume(resume_id_value, user_id_value)?;

    let is_owner = match user_id_value {
        Some(uid) => match find_resume(resume_id_value) {
            Ok(resume) => resume.created_by == Some(uid),
            Err(_) => false,
        },
        None => false,
    };

    let mut education_query = education_dsl::education.into_boxed();
    education_query = education_query
        .filter(education_dsl::id.eq(education_id_value))
        .filter(education_dsl::resume_id.eq(resume_id_value));
    if !is_owner {
        education_query = education_query.filter(education_dsl::active.eq(true));
    }

    let _education: Education = match education_query.first(&mut establish_connection()) {
        Ok(e) => e,
        Err(err) => {
            return Err(app_err_from_diesel_err(err));
        }
    };

    let mut kp_query = key_points_dsl::education_key_points.into_boxed();
    kp_query = kp_query.filter(key_points_dsl::education_id.eq(education_id_value));
    if !is_owner {
        kp_query = kp_query.filter(key_points_dsl::active.eq(true));
    }

    let mut items: Vec<EducationKeyPoint> =
        match kp_query.load::<EducationKeyPoint>(&mut establish_connection()) {
            Ok(v) => v,
            Err(err) => {
                return Err(app_err_from_diesel_err(err));
            }
        };

    items.sort_by_key(|kp| (kp.display_order.unwrap_or(0), kp.id));

    Ok(items)
}
