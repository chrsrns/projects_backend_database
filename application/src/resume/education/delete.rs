use diesel::prelude::*;
use domain::models::{Education, EducationKeyPoint, Resume};
use infrastructure::establish_connection;
use rocket::http::Status;
use rocket::response::status::{Custom, NoContent};
use shared::response_models::{Response, ResponseBody};

pub fn delete_education(
    user_id_value: i32,
    education_id_value: i32,
) -> Result<NoContent, Custom<String>> {
    use domain::schema::education;
    use domain::schema::resumes;

    let existing: Education = match education::table
        .find(education_id_value)
        .first(&mut establish_connection())
    {
        Ok(v) => v,
        Err(diesel::result::Error::NotFound) => {
            let response = Response {
                body: ResponseBody::Message(format!(
                    "Education with id {} not found",
                    education_id_value
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

    match diesel::delete(education::table.find(education_id_value))
        .execute(&mut establish_connection())
    {
        Ok(count) => {
            if count == 0 {
                let response = Response {
                    body: ResponseBody::Message(format!(
                        "Education with id {} not found",
                        education_id_value
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

pub fn delete_education_key_point(
    user_id_value: i32,
    key_point_id_value: i32,
) -> Result<NoContent, Custom<String>> {
    use domain::schema::education;
    use domain::schema::education_key_points;
    use domain::schema::resumes;

    let existing: EducationKeyPoint = match education_key_points::table
        .find(key_point_id_value)
        .first(&mut establish_connection())
    {
        Ok(v) => v,
        Err(diesel::result::Error::NotFound) => {
            let response = Response {
                body: ResponseBody::Message(format!(
                    "Education key point with id {} not found",
                    key_point_id_value
                )),
            };
            return Err(Custom(
                Status::NotFound,
                serde_json::to_string(&response).unwrap(),
            ));
        }
        Err(err) => panic!("Database error - {}", err),
    };

    let edu: Education = match education::table
        .find(existing.education_id)
        .first(&mut establish_connection())
    {
        Ok(e) => e,
        Err(diesel::result::Error::NotFound) => {
            let response = Response {
                body: ResponseBody::Message("Education not found".to_string()),
            };
            return Err(Custom(
                Status::NotFound,
                serde_json::to_string(&response).unwrap(),
            ));
        }
        Err(err) => panic!("Database error - {}", err),
    };

    let resume: Resume = match resumes::table
        .find(edu.resume_id)
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

    match diesel::delete(education_key_points::table.find(key_point_id_value))
        .execute(&mut establish_connection())
    {
        Ok(count) => {
            if count == 0 {
                let response = Response {
                    body: ResponseBody::Message(format!(
                        "Education key point with id {} not found",
                        key_point_id_value
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
