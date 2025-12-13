#[macro_use]
extern crate rocket;

pub mod auth;
pub mod auth_handler;
pub mod resume_handler;

pub fn build_rocket() -> rocket::Rocket<rocket::Build> {
    rocket::build().mount(
        "/api",
        routes![
            auth_handler::register_handler,
            auth_handler::login_handler,
            auth_handler::me_handler,
            auth_handler::logout_handler,
            resume_handler::list_resumes_handler,
            resume_handler::list_resume_handler,
            resume_handler::create_resume_handler,
            resume_handler::update_resume_handler,
            resume_handler::delete_resume_handler,
        ],
    )
}
