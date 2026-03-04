use diesel::prelude::*;
use domain::models::{WorkExperience, WorkExperienceKeyPoint};
use infrastructure::establish_connection;

use crate::{
    error::ApplicationError,
    resume::common::{app_err_from_diesel_err, find_accessible_resume},
};

pub fn list_work_experiences(
    resume_id_value: i32,
    user_id_value: Option<i32>,
) -> Result<Vec<WorkExperience>, ApplicationError> {
    use domain::schema::work_experiences::dsl as work_dsl;

    if let Err(err) = find_accessible_resume(resume_id_value, user_id_value) {
        return Err(err);
    }

    let mut items: Vec<WorkExperience> = match work_dsl::work_experiences
        .filter(work_dsl::resume_id.eq(resume_id_value))
        .load::<WorkExperience>(&mut establish_connection())
    {
        Ok(v) => v,
        Err(err) => return Err(app_err_from_diesel_err(err)),
    };

    items.sort_by_key(|w| (w.display_order.unwrap_or(0), w.id));

    Ok(items)
}

pub fn list_work_experience_key_points(
    resume_id_value: i32,
    work_id_value: i32,
    user_id_value: Option<i32>,
) -> Result<Vec<WorkExperienceKeyPoint>, ApplicationError> {
    use domain::schema::work_experience_key_points::dsl as kps_dsl;
    use domain::schema::work_experiences::dsl as work_dsl;

    if let Err(err) = find_accessible_resume(resume_id_value, user_id_value) {
        return Err(err);
    }

    let _work: WorkExperience = match work_dsl::work_experiences
        .filter(work_dsl::id.eq(work_id_value))
        .filter(work_dsl::resume_id.eq(resume_id_value))
        .first(&mut establish_connection())
    {
        Ok(w) => w,
        Err(diesel::result::Error::NotFound) => {
            return Err(ApplicationError::NotFound(
                "Work experience not found".to_string(),
            ));
        }
        Err(err) => return Err(app_err_from_diesel_err(err)),
    };

    let mut items: Vec<WorkExperienceKeyPoint> = match kps_dsl::work_experience_key_points
        .filter(kps_dsl::work_experience_id.eq(work_id_value))
        .load::<WorkExperienceKeyPoint>(&mut establish_connection())
    {
        Ok(v) => v,
        Err(err) => return Err(app_err_from_diesel_err(err)),
    };

    items.sort_by_key(|kp| (kp.display_order.unwrap_or(0), kp.id));

    Ok(items)
}
