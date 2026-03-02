use diesel::prelude::*;
use domain::models::Resume;
use infrastructure::establish_connection;

use crate::error::ApplicationError;

pub fn list_resume(resume_id: i32, user_id_value: Option<i32>) -> Result<Resume, ApplicationError> {
    use domain::schema::resumes;
    use domain::schema::resumes::dsl::*;

    let mut query = resumes::table.into_boxed();
    query = query.filter(resumes::id.eq(resume_id));
    query = match user_id_value {
        Some(uid) => query.filter(is_public.eq(true).or(created_by.eq(uid))),
        None => query.filter(is_public.eq(true)),
    };

    match query.first::<Resume>(&mut establish_connection()) {
        Ok(resume) => Ok(resume),
        Err(err) => match err {
            diesel::result::Error::NotFound => Err(ApplicationError::NotFound(format!(
                "Error selecting resume with id {} - {}",
                resume_id, err
            ))),
            _ => Err(ApplicationError::Internal(format!(
                "Database error - {}",
                err
            ))),
        },
    }
}

pub fn list_resumes(user_id_value: Option<i32>) -> Result<Vec<Resume>, ApplicationError> {
    use domain::schema::resumes;
    use domain::schema::resumes::dsl::*;

    let mut query = resumes.into_boxed();
    query = match user_id_value {
        Some(uid) => query.filter(is_public.eq(true).or(created_by.eq(uid))),
        None => query.filter(is_public.eq(true)),
    };

    match query
        .select(resumes::all_columns)
        .load::<Resume>(&mut establish_connection())
    {
        Ok(mut items) => {
            items.sort();
            Ok(items)
        }
        Err(err) => Err(ApplicationError::Internal(format!(
            "Database error - {}",
            err
        ))),
    }
}
