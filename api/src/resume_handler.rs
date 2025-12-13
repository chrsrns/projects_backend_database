use application::resume::{create, delete, read, update};
use domain::models::{NewResume, Resume, UpdateResume};
use rocket::response::status::{Conflict, Created, NoContent, NotFound};
use rocket::serde::json::Json;
use rocket::{delete as rocket_delete, get, post, put};
use shared::response_models::{Response, ResponseBody};

#[get("/resumes")]
pub fn list_resumes_handler() -> String {
    let resumes: Vec<Resume> = read::list_resumes();
    let response = Response {
        body: ResponseBody::Resumes(resumes),
    };

    serde_json::to_string(&response).unwrap()
}

#[get("/resume/<resume_id>")]
pub fn list_resume_handler(resume_id: i32) -> Result<String, NotFound<String>> {
    let resume = read::list_resume(resume_id)?;
    let response = Response {
        body: ResponseBody::Resume(resume),
    };

    Ok(serde_json::to_string(&response).unwrap())
}

#[post("/new_resume", format = "application/json", data = "<resume>")]
pub fn create_resume_handler(resume: Json<NewResume>) -> Result<Created<String>, Conflict<String>> {
    create::create_resume(resume)
}

#[put("/resume/<resume_id>", format = "application/json", data = "<resume>")]
pub fn update_resume_handler(
    resume_id: i32,
    resume: Json<UpdateResume>,
) -> Result<String, NotFound<String>> {
    update::update_resume(resume_id, resume)
}

#[rocket_delete("/resume/<resume_id>")]
pub fn delete_resume_handler(resume_id: i32) -> Result<NoContent, NotFound<String>> {
    delete::delete_resume(resume_id)
}
