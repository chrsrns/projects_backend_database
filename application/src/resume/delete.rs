use diesel::prelude::*;
use infrastructure::establish_connection;
use rocket::response::status::{NoContent, NotFound};
use shared::response_models::{Response, ResponseBody};

pub fn delete_resume(resume_id: i32) -> Result<NoContent, NotFound<String>> {
    use domain::schema::resumes;

    match diesel::delete(resumes::table.find(resume_id)).execute(&mut establish_connection()) {
        Ok(count) => {
            if count == 0 {
                let response = Response {
                    body: ResponseBody::Message(format!("Resume with id {} not found", resume_id)),
                };
                Err(NotFound(serde_json::to_string(&response).unwrap()))
            } else {
                Ok(NoContent)
            }
        }
        Err(err) => {
            panic!("Database error - {}", err);
        }
    }
}
