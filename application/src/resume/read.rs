use domain::models::Resume;
use shared::response_models::{Response, ResponseBody};
use infrastructure::establish_connection;
use diesel::prelude::*;
use rocket::response::status::NotFound;

pub fn list_resume(resume_id: i32) -> Result<Resume, NotFound<String>> {
    use domain::schema::resumes;

    match resumes::table.find(resume_id).first::<Resume>(&mut establish_connection()) {
        Ok(resume) => Ok(resume),
        Err(err) => match err {
            diesel::result::Error::NotFound => {
                let response = Response { body: ResponseBody::Message(format!("Error selecting resume with id {} - {}", resume_id, err))};
                return Err(NotFound(serde_json::to_string(&response).unwrap()));
            },
            _ => {
                panic!("Database error - {}", err);
            }        
        }
    }
}

pub fn list_resumes() -> Vec<Resume> {
    use domain::schema::resumes;

    match resumes::table.select(resumes::all_columns).load::<Resume>(&mut establish_connection()) {
        Ok(mut resumes) => {
            resumes.sort();
            resumes
        },
        Err(err) => match err {
            _ => {
                panic!("Database error - {}", err);
            }
        }
    }
}