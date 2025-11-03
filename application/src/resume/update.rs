use domain::models::{Resume, UpdateResume};
use shared::response_models::{Response, ResponseBody};
use infrastructure::establish_connection;
use diesel::prelude::*;
use rocket::response::status::NotFound;
use rocket::serde::json::Json;

pub fn update_resume(resume_id: i32, resume: Json<UpdateResume>) -> Result<String, NotFound<String>> {
    use domain::schema::resumes;

    let resume = resume.into_inner();

    match diesel::update(resumes::table.find(resume_id))
        .set(&resume)
        .get_result::<Resume>(&mut establish_connection()) {
        Ok(updated_resume) => {
            let response = Response { body: ResponseBody::Resume(updated_resume) };
            Ok(serde_json::to_string(&response).unwrap())
        },
        Err(err) => match err {
            diesel::result::Error::NotFound => {
                let response = Response { 
                    body: ResponseBody::Message(format!("Error updating resume with id {} - {}", resume_id, err))
                };
                Err(NotFound(serde_json::to_string(&response).unwrap()))
            },
            _ => {
                panic!("Database error - {}", err);
            }
        }
    }
}
