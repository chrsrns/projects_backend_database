use diesel::prelude::*;
use domain::models::{Language, Resume, UpdateLanguage};
use infrastructure::establish_connection;
use rocket::http::Status;
use rocket::response::status::Custom;
use rocket::serde::json::Json;
use shared::response_models::{Response, ResponseBody};

pub fn update_language(
    user_id_value: i32,
    language_id_value: i32,
    payload: Json<UpdateLanguage>,
) -> Result<String, Custom<String>> {
    use domain::schema::languages;
    use domain::schema::resumes;

    let existing: Language = match languages::table
        .find(language_id_value)
        .first(&mut establish_connection())
    {
        Ok(r) => r,
        Err(diesel::result::Error::NotFound) => {
            let response = Response {
                body: ResponseBody::Message(format!(
                    "Language with id {} not found",
                    language_id_value
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

    let payload = payload.into_inner();

    match diesel::update(languages::table.find(language_id_value))
        .set(&payload)
        .get_result::<Language>(&mut establish_connection())
    {
        Ok(updated) => {
            let response = Response {
                body: ResponseBody::Language(updated),
            };
            Ok(serde_json::to_string(&response).unwrap())
        }
        Err(diesel::result::Error::NotFound) => {
            let response = Response {
                body: ResponseBody::Message(format!(
                    "Language with id {} not found",
                    language_id_value
                )),
            };
            Err(Custom(
                Status::NotFound,
                serde_json::to_string(&response).unwrap(),
            ))
        }
        Err(err) => panic!("Database error - {}", err),
    }
}
