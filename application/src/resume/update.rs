use diesel::prelude::*;
use domain::models::{Resume, UpdateResume};
use infrastructure::establish_connection;
use rocket::http::Status;
use rocket::response::status::Custom;
use rocket::serde::json::Json;
use shared::response_models::Response;

pub fn update_resume(
    user_id_value: i32,
    resume_id: i32,
    resume: Json<UpdateResume>,
) -> Result<String, Custom<String>> {
    use domain::schema::resumes;

    let existing: Resume = match resumes::table
        .find(resume_id)
        .first(&mut establish_connection())
    {
        Ok(r) => r,
        Err(diesel::result::Error::NotFound) => {
            let response = Response::<String> {
                body: format!(
                    "Error updating resume with id {} - {}",
                    resume_id,
                    diesel::result::Error::NotFound
                ),
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
            let response = Response::<String> {
                body: "Forbidden".to_string(),
            };
            return Err(Custom(
                Status::Forbidden,
                serde_json::to_string(&response).unwrap(),
            ));
        }
    }

    let resume = resume.into_inner();

    match diesel::update(resumes::table.find(resume_id))
        .set(&resume)
        .get_result::<Resume>(&mut establish_connection())
    {
        Ok(updated_resume) => {
            let response = Response::<Resume> {
                body: updated_resume,
            };
            Ok(serde_json::to_string(&response).unwrap())
        }
        Err(err) => match err {
            diesel::result::Error::NotFound => {
                let response = Response::<String> {
                    body: format!("Error updating resume with id {} - {}", resume_id, err),
                };
                Err(Custom(
                    Status::NotFound,
                    serde_json::to_string(&response).unwrap(),
                ))
            }
            _ => {
                panic!("Database error - {}", err);
            }
        },
    }
}
