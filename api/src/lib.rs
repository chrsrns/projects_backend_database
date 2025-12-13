#[macro_use]
extern crate rocket;

pub mod resume_handler;

pub fn build_rocket() -> rocket::Rocket<rocket::Build> {
    rocket::build().mount(
        "/api",
        routes![
            resume_handler::list_resumes_handler,
            resume_handler::list_resume_handler,
            resume_handler::create_resume_handler,
            resume_handler::update_resume_handler,
            resume_handler::delete_resume_handler,
        ],
    )
}
