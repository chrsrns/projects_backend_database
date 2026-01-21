use diesel::prelude::*;
use domain::models::{Resume, Skill};
use infrastructure::establish_connection;
use rocket::http::Status;
use rocket::response::status::{Custom, NoContent};
use shared::response_models::{Response, ResponseBody};

pub fn delete_skill(user_id_value: i32, skill_id_value: i32) -> Result<NoContent, Custom<String>> {
    use domain::schema::resumes;
    use domain::schema::skills;

    let existing: Skill = match skills::table
        .find(skill_id_value)
        .first(&mut establish_connection())
    {
        Ok(r) => r,
        Err(diesel::result::Error::NotFound) => {
            let response = Response {
                body: ResponseBody::Message(format!("Skill with id {} not found", skill_id_value)),
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

    match diesel::delete(skills::table.find(skill_id_value)).execute(&mut establish_connection()) {
        Ok(count) => {
            if count == 0 {
                let response = Response {
                    body: ResponseBody::Message(format!(
                        "Skill with id {} not found",
                        skill_id_value
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
