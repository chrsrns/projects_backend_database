use domain::models::{Resume, NewResume};
use shared::response_models::{Response, ResponseBody};
use infrastructure::establish_connection;
use diesel::prelude::*;
use rocket::response::status::{Created, Conflict};
use rocket::serde::json::Json;

pub fn create_resume(resume: Json<NewResume>) -> Result<Created<String>, Conflict<String>> {
    use domain::schema::resumes;

    let resume = resume.into_inner();

    match diesel::insert_into(resumes::table).values(&resume).get_result::<Resume>(&mut establish_connection()) {
        Ok(resume) => {
            let response = Response { body: ResponseBody::Resume(resume) };
            Ok(Created::new("").tagged_body(serde_json::to_string(&response).unwrap()))
        },
        Err(err) => match err {
            diesel::result::Error::DatabaseError(diesel::result::DatabaseErrorKind::UniqueViolation, _) => {
                let response = Response {
                    body: ResponseBody::Message("Resume with this email already exists".to_string()),
                };
                Err(Conflict(serde_json::to_string(&response).unwrap()))
            },
            _ => panic!("Database error - {}", err),
        }
    }
}