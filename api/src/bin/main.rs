#[macro_use] extern crate rocket;
use api::resume_handler;

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/api", routes![
            resume_handler::list_resumes_handler, 
            resume_handler::list_resume_handler,
            resume_handler::create_resume_handler,
            resume_handler::update_resume_handler,
            resume_handler::delete_resume_handler,
        ])
}