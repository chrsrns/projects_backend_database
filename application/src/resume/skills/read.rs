use diesel::prelude::*;
use domain::models::{Resume, Skill};
use infrastructure::establish_connection;

use crate::{error::ApplicationError, resume::common::app_err_from_diesel_err};

pub fn list_skills(
    resume_id_value: i32,
    user_id_value: Option<i32>,
) -> Result<Vec<Skill>, ApplicationError> {
    use domain::schema::resumes::dsl as resumes_dsl;
    use domain::schema::skills::dsl as skills_dsl;

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

    let mut items: Vec<Skill> = match skills_dsl::skills
        .filter(skills_dsl::resume_id.eq(resume_id_value))
        .load::<Skill>(&mut establish_connection())
    {
        Ok(v) => v,
        Err(err) => return Err(app_err_from_diesel_err(err)),
    };

    items.sort_by_key(|s| (s.display_order.unwrap_or(0), s.id));

    Ok(items)
}
