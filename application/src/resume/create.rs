use domain::models::{Resume, NewResume};
use shared::response_models::{Response, ResponseBody};
use infrastructure::establish_connection;
use diesel::prelude::*;
use rocket::response::status::Created;
use rocket::serde::json::Json;

pub fn create_resume(resume: Json<NewResume>) -> Created<String> {
    use domain::schema::resumes;

    let resume = resume.into_inner();

    match diesel::insert_into(resumes::table).values(&resume).get_result::<Resume>(&mut establish_connection()) {
        Ok(resume) => {
            let response = Response { body: ResponseBody::Resume(resume) };
            Created::new("").tagged_body(serde_json::to_string(&response).unwrap())
        },
        Err(err) => match err {
            _ => {
                panic!("Database error - {}", err);
            }
        }
    }
}