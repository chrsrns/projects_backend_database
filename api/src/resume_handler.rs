use application::resume::{create, delete, read, update};
use domain::models::{NewResumeRequest, Resume, UpdateResume};
use rocket::response::status::{Conflict, Created, NoContent, NotFound};
use rocket::serde::json::Json;
use rocket::{delete as rocket_delete, get, post, put};
use shared::response_models::Response;

use crate::auth::{AuthSession, MaybeAuthSession};

#[get("/resumes")]
pub fn list_resumes_handler(maybe_auth: MaybeAuthSession) -> String {
    let user_id_value = maybe_auth.0.map(|a| a.user_id);
    let resumes: Vec<Resume> = read::list_resumes(user_id_value);
    let response = Response { body: resumes };

    serde_json::to_string(&response).unwrap()
}

#[get("/resume/<resume_id>")]
pub fn list_resume_handler(
    resume_id: i32,
    maybe_auth: MaybeAuthSession,
) -> Result<String, NotFound<String>> {
    let user_id_value = maybe_auth.0.map(|a| a.user_id);
    let resume = read::list_resume(resume_id, user_id_value)?;
    let response = Response { body: resume };

    Ok(serde_json::to_string(&response).unwrap())
}

#[post("/new_resume", format = "application/json", data = "<resume>")]
pub fn create_resume_handler(
    auth: AuthSession,
    resume: Json<NewResumeRequest>,
) -> Result<Created<String>, Conflict<String>> {
    create::create_resume(auth.user_id, resume)
}

#[put("/resume/<resume_id>", format = "application/json", data = "<resume>")]
pub fn update_resume_handler(
    auth: AuthSession,
    resume_id: i32,
    resume: Json<UpdateResume>,
) -> Result<String, rocket::response::status::Custom<String>> {
    update::update_resume(auth.user_id, resume_id, resume)
}

#[rocket_delete("/resume/<resume_id>")]
pub fn delete_resume_handler(
    auth: AuthSession,
    resume_id: i32,
) -> Result<NoContent, rocket::response::status::Custom<String>> {
    delete::delete_resume(auth.user_id, resume_id)
}
