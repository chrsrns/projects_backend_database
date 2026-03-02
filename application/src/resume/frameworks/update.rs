use diesel::prelude::*;
use domain::models::{Framework, Language, Resume, UpdateFramework};
use infrastructure::establish_connection;
use rocket::http::Status;
use rocket::response::status::Custom;
use rocket::serde::json::Json;
use shared::response_models::Response;

pub fn update_framework(
    user_id_value: i32,
    framework_id_value: i32,
    payload: Json<UpdateFramework>,
) -> Result<String, Custom<String>> {
    use domain::schema::frameworks;
    use domain::schema::languages;
    use domain::schema::resumes;

    let existing: Framework = match frameworks::table
        .find(framework_id_value)
        .first(&mut establish_connection())
    {
        Ok(r) => r,
        Err(diesel::result::Error::NotFound) => {
            let response = Response::<String> {
                body: format!("Framework with id {} not found", framework_id_value),
            };
            return Err(Custom(
                Status::NotFound,
                serde_json::to_string(&response).unwrap(),
            ));
        }
        Err(err) => panic!("Database error - {}", err),
    };

    let language: Language = match languages::table
        .find(existing.language_id)
        .first(&mut establish_connection())
    {
        Ok(l) => l,
        Err(diesel::result::Error::NotFound) => {
            let response = Response::<String> {
                body: "Language not found".to_string(),
            };
            return Err(Custom(
                Status::NotFound,
                serde_json::to_string(&response).unwrap(),
            ));
        }
        Err(err) => panic!("Database error - {}", err),
    };

    let resume: Resume = match resumes::table
        .find(language.resume_id)
        .first(&mut establish_connection())
    {
        Ok(r) => r,
        Err(diesel::result::Error::NotFound) => {
            let response = Response::<String> {
                body: "Resume not found".to_string(),
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
            let response = Response::<String> {
                body: "Forbidden".to_string(),
            };
            return Err(Custom(
                Status::Forbidden,
                serde_json::to_string(&response).unwrap(),
            ));
        }
    }

    let payload = payload.into_inner();

    match diesel::update(frameworks::table.find(framework_id_value))
        .set(&payload)
        .get_result::<Framework>(&mut establish_connection())
    {
        Ok(updated) => {
            let response = Response::<Framework> { body: updated };
            Ok(serde_json::to_string(&response).unwrap())
        }
        Err(diesel::result::Error::NotFound) => {
            let response = Response::<String> {
                body: format!("Framework with id {} not found", framework_id_value),
            };
            Err(Custom(
                Status::NotFound,
                serde_json::to_string(&response).unwrap(),
            ))
        }
        Err(err) => panic!("Database error - {}", err),
    }
}
