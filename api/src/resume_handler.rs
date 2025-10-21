use shared::response_models::{Response, ResponseBody};
use application::resume::{create, read};
use domain::models::{Resume, NewResume};
use rocket::{get, post};
use rocket::response::status::{NotFound, Created};
use rocket::serde::json::Json;

#[get("/")]
pub fn list_resumes_handler() -> String {
    let resumes: Vec<Resume> = read::list_resumes();
    let response = Response { body: ResponseBody::Resumes(resumes) };

    serde_json::to_string(&response).unwrap()
}

#[get("/resume/<resume_id>")]
pub fn list_resume_handler(resume_id: i32) -> Result<String, NotFound<String>> {
    let resume = read::list_resume(resume_id)?;
    let response = Response { body: ResponseBody::Resume(resume) };

    Ok(serde_json::to_string(&response).unwrap())
}

#[post("/new_resume", format = "application/json", data = "<resume>")]
pub fn create_resume_handler(resume: Json<NewResume>) -> Created<String> {
    create::create_resume(resume)
}