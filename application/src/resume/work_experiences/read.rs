use diesel::prelude::*;
use domain::models::{Resume, WorkExperience, WorkExperienceKeyPoint};
use infrastructure::establish_connection;
use rocket::response::status::NotFound;
use shared::response_models::Response;

pub fn list_work_experiences(
    resume_id_value: i32,
    user_id_value: Option<i32>,
) -> Result<String, NotFound<String>> {
    use domain::schema::resumes::dsl as resumes_dsl;
    use domain::schema::work_experiences::dsl as work_dsl;

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
        Err(diesel::result::Error::NotFound) => {
            let response = Response::<String> {
                body: format!("Resume with id {} not found", resume_id_value),
            };
            return Err(NotFound(serde_json::to_string(&response).unwrap()));
        }
        Err(err) => panic!("Database error - {}", err),
    };

    let mut items: Vec<WorkExperience> = match work_dsl::work_experiences
        .filter(work_dsl::resume_id.eq(resume_id_value))
        .load::<WorkExperience>(&mut establish_connection())
    {
        Ok(v) => v,
        Err(err) => panic!("Database error - {}", err),
    };

    items.sort_by_key(|w| (w.display_order.unwrap_or(0), w.id));

    let response = Response::<Vec<WorkExperience>> { body: items };

    Ok(serde_json::to_string(&response).unwrap())
}

pub fn list_work_experience_key_points(
    resume_id_value: i32,
    work_id_value: i32,
    user_id_value: Option<i32>,
) -> Result<String, NotFound<String>> {
    use domain::schema::resumes::dsl as resumes_dsl;
    use domain::schema::work_experience_key_points::dsl as kps_dsl;
    use domain::schema::work_experiences::dsl as work_dsl;

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
        Err(diesel::result::Error::NotFound) => {
            let response = Response::<String> {
                body: format!("Resume with id {} not found", resume_id_value),
            };
            return Err(NotFound(serde_json::to_string(&response).unwrap()));
        }
        Err(err) => panic!("Database error - {}", err),
    };

    let _work: WorkExperience = match work_dsl::work_experiences
        .filter(work_dsl::id.eq(work_id_value))
        .filter(work_dsl::resume_id.eq(resume_id_value))
        .first(&mut establish_connection())
    {
        Ok(w) => w,
        Err(diesel::result::Error::NotFound) => {
            let response = Response::<String> {
                body: "Work experience not found".to_string(),
            };
            return Err(NotFound(serde_json::to_string(&response).unwrap()));
        }
        Err(err) => panic!("Database error - {}", err),
    };

    let mut items: Vec<WorkExperienceKeyPoint> = match kps_dsl::work_experience_key_points
        .filter(kps_dsl::work_experience_id.eq(work_id_value))
        .load::<WorkExperienceKeyPoint>(&mut establish_connection())
    {
        Ok(v) => v,
        Err(err) => panic!("Database error - {}", err),
    };

    items.sort_by_key(|kp| (kp.display_order.unwrap_or(0), kp.id));

    let response = Response::<Vec<WorkExperienceKeyPoint>> { body: items };

    Ok(serde_json::to_string(&response).unwrap())
}
