use diesel::prelude::*;
use domain::models::{Resume, WorkExperience, WorkExperienceKeyPoint};
use infrastructure::establish_connection;
use rocket::http::Status;
use rocket::response::status::{Custom, NoContent};
use shared::response_models::{Response, ResponseBody};

pub fn delete_work_experience(
    user_id_value: i32,
    work_id_value: i32,
) -> Result<NoContent, Custom<String>> {
    use domain::schema::resumes;
    use domain::schema::work_experiences;

    let existing: WorkExperience = match work_experiences::table
        .find(work_id_value)
        .first(&mut establish_connection())
    {
        Ok(v) => v,
        Err(diesel::result::Error::NotFound) => {
            let response = Response {
                body: ResponseBody::Message(format!(
                    "Work experience with id {} not found",
                    work_id_value
                )),
            };
            return Err(Custom(
                Status::NotFound,
                serde_json::to_string(&response).unwrap(),
            ));
        }
        Err(err) => panic!("Database error - {}", err),
    };

    let resume: Resume = match resumes::table
        .find(existing.resume_id)
        .first(&mut establish_connection())
    {
        Ok(r) => r,
        Err(diesel::result::Error::NotFound) => {
            let response = Response {
                body: ResponseBody::Message("Resume not found".to_string()),
            };
            return Err(Custom(
                Status::NotFound,
                serde_json::to_string(&response).unwrap(),
            ));
        }
        Err(err) => panic!("Database error - {}", err),
    };

    match resume.created_by {
        Some(owner) if owner == user_id_value => {}
        Some(_) | None => {
            let response = Response {
                body: ResponseBody::Message("Forbidden".to_string()),
            };
            return Err(Custom(
                Status::Forbidden,
                serde_json::to_string(&response).unwrap(),
            ));
        }
    }

    match diesel::delete(work_experiences::table.find(work_id_value))
        .execute(&mut establish_connection())
    {
        Ok(count) => {
            if count == 0 {
                let response = Response {
                    body: ResponseBody::Message(format!(
                        "Work experience with id {} not found",
                        work_id_value
                    )),
                };
                Err(Custom(
                    Status::NotFound,
                    serde_json::to_string(&response).unwrap(),
                ))
            } else {
                Ok(NoContent)
            }
        }
        Err(err) => panic!("Database error - {}", err),
    }
}

pub fn delete_work_experience_key_point(
    user_id_value: i32,
    kp_id_value: i32,
) -> Result<NoContent, Custom<String>> {
    use domain::schema::resumes;
    use domain::schema::work_experience_key_points;
    use domain::schema::work_experiences;

    let existing: WorkExperienceKeyPoint = match work_experience_key_points::table
        .find(kp_id_value)
        .first(&mut establish_connection())
    {
        Ok(v) => v,
        Err(diesel::result::Error::NotFound) => {
            let response = Response {
                body: ResponseBody::Message(format!(
                    "Work experience key point with id {} not found",
                    kp_id_value
                )),
            };
            return Err(Custom(
                Status::NotFound,
                serde_json::to_string(&response).unwrap(),
            ));
        }
        Err(err) => panic!("Database error - {}", err),
    };

    let work: WorkExperience = match work_experiences::table
        .find(existing.work_experience_id)
        .first(&mut establish_connection())
    {
        Ok(w) => w,
        Err(diesel::result::Error::NotFound) => {
            let response = Response {
                body: ResponseBody::Message("Work experience not found".to_string()),
            };
            return Err(Custom(
                Status::NotFound,
                serde_json::to_string(&response).unwrap(),
            ));
        }
        Err(err) => panic!("Database error - {}", err),
    };

    let resume: Resume = match resumes::table
        .find(work.resume_id)
        .first(&mut establish_connection())
    {
        Ok(r) => r,
        Err(diesel::result::Error::NotFound) => {
            let response = Response {
                body: ResponseBody::Message("Resume not found".to_string()),
            };
            return Err(Custom(
                Status::NotFound,
                serde_json::to_string(&response).unwrap(),
            ));
        }
        Err(err) => panic!("Database error - {}", err),
    };

    match resume.created_by {
        Some(owner) if owner == user_id_value => {}
        Some(_) | None => {
            let response = Response {
                body: ResponseBody::Message("Forbidden".to_string()),
            };
            return Err(Custom(
                Status::Forbidden,
                serde_json::to_string(&response).unwrap(),
            ));
        }
    }

    match diesel::delete(work_experience_key_points::table.find(kp_id_value))
        .execute(&mut establish_connection())
    {
        Ok(count) => {
            if count == 0 {
                let response = Response {
                    body: ResponseBody::Message(format!(
                        "Work experience key point with id {} not found",
                        kp_id_value
                    )),
                };
                Err(Custom(
                    Status::NotFound,
                    serde_json::to_string(&response).unwrap(),
                ))
            } else {
                Ok(NoContent)
            }
        }
        Err(err) => panic!("Database error - {}", err),
    }
}
