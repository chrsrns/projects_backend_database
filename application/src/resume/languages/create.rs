use diesel::prelude::*;
use domain::models::{Language, NewLanguage, NewLanguageRequest, Resume};
use infrastructure::establish_connection;
use rocket::http::Status;
use rocket::response::status::{Created, Custom};
use rocket::serde::json::Json;
use shared::response_models::Response;

pub fn create_language(
    user_id_value: i32,
    resume_id_value: i32,
    payload: Json<NewLanguageRequest>,
) -> Result<Created<String>, Custom<String>> {
    use domain::schema::languages;
    use domain::schema::resumes;

    let existing_resume: Resume = match resumes::table
        .find(resume_id_value)
        .first(&mut establish_connection())
    {
        Ok(r) => r,
        Err(diesel::result::Error::NotFound) => {
            let response = Response::<String> {
                body: format!("Resume with id {} not found", resume_id_value),
            };
            return Err(Custom(
                Status::NotFound,
                serde_json::to_string(&response).unwrap(),
            ));
        }
        Err(err) => panic!("Database error - {}", err),
    };

    match existing_resume.created_by {
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
    let new_language = NewLanguage {
        resume_id: resume_id_value,
        language_name: payload.language_name,
        display_order: payload.display_order,
    };

    match diesel::insert_into(languages::table)
        .values(&new_language)
        .get_result::<Language>(&mut establish_connection())
    {
        Ok(language) => {
            let response = Response::<Language> { body: language };
            Ok(Created::new("").tagged_body(serde_json::to_string(&response).unwrap()))
        }
        Err(err) => {
            panic!("Database error - {}", err);
        }
    }
}
