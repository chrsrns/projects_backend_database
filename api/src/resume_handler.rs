use shared::response_models::{Response, ResponseBody};
use application::resume::{read};
use domain::models::{Resume};
use rocket::{get};
use rocket::response::status::{NotFound};
use rocket::serde::json::Json;

#[get("/")]
pub fn list_resumes_handler() -> String {
    todo!()
}

#[get("/resume/<resume_id>")]
pub fn list_resume_handler(resume_id: i32) -> Result<String, NotFound<String>> {
    todo!()
}