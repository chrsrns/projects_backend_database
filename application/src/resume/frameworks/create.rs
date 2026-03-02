use diesel::prelude::*;
use domain::models::{Framework, Language, NewFramework, NewFrameworkRequest, Resume};
use infrastructure::establish_connection;
use rocket::http::Status;
use rocket::response::status::{Created, Custom};
use rocket::serde::json::Json;
use shared::response_models::Response;

pub fn create_framework(
    user_id_value: i32,
    resume_id_value: i32,
    language_id_value: i32,
    payload: Json<NewFrameworkRequest>,
) -> Result<Created<String>, Custom<String>> {
    use domain::schema::frameworks;
    use domain::schema::languages::dsl as languages_dsl;
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

    let _language: Language = match languages_dsl::languages
        .filter(languages_dsl::id.eq(language_id_value))
        .filter(languages_dsl::resume_id.eq(resume_id_value))
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

    let payload = payload.into_inner();
    let new_framework = NewFramework {
        language_id: language_id_value,
        framework_name: payload.framework_name,
        display_order: payload.display_order,
    };

    match diesel::insert_into(frameworks::table)
        .values(&new_framework)
        .get_result::<Framework>(&mut establish_connection())
    {
        Ok(framework) => {
            let response = Response::<Framework> { body: framework };
            Ok(Created::new("").tagged_body(serde_json::to_string(&response).unwrap()))
        }
        Err(err) => panic!("Database error - {}", err),
    }
}
