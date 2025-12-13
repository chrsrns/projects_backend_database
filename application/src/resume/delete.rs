use diesel::prelude::*;
use domain::models::Resume;
use infrastructure::establish_connection;
use rocket::http::Status;
use rocket::response::status::{Custom, NoContent};
use shared::response_models::{Response, ResponseBody};

pub fn delete_resume(user_id_value: i32, resume_id: i32) -> Result<NoContent, Custom<String>> {
    use domain::schema::resumes;

    let existing: Resume = match resumes::table
        .find(resume_id)
        .first(&mut establish_connection())
    {
        Ok(r) => r,
        Err(diesel::result::Error::NotFound) => {
            let response = Response {
                body: ResponseBody::Message(format!("Resume with id {} not found", resume_id)),
            };
            return Err(Custom(
                Status::NotFound,
                serde_json::to_string(&response).unwrap(),
            ));
        }
        Err(err) => panic!("Database error - {}", err),
    };

    match existing.created_by {
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

    match diesel::delete(resumes::table.find(resume_id)).execute(&mut establish_connection()) {
        Ok(count) => {
            if count == 0 {
                let response = Response {
                    body: ResponseBody::Message(format!("Resume with id {} not found", resume_id)),
                };
                Err(Custom(
                    Status::NotFound,
                    serde_json::to_string(&response).unwrap(),
                ))
            } else {
                Ok(NoContent)
            }
        }
        Err(err) => {
            panic!("Database error - {}", err);
        }
    }
}
