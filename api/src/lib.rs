#[macro_use]
extern crate rocket;

use utoipa::OpenApi;

pub mod auth;
pub mod openapi;
pub mod realtime;
pub mod route_handlers;
pub mod ws_handler;

use route_handlers::resume::*;

pub fn build_rocket() -> rocket::Rocket<rocket::Build> {
    build_rocket_with_hub(realtime::Hub::new())
}

pub fn build_rocket_with_hub(hub: realtime::Hub) -> rocket::Rocket<rocket::Build> {
    let allowed_origins = rocket_cors::AllowedOrigins::all();

    let cors = rocket_cors::CorsOptions {
        allowed_origins,
        allowed_methods: vec![
            rocket::http::Method::Get,
            rocket::http::Method::Post,
            rocket::http::Method::Put,
            rocket::http::Method::Delete,
        ]
        .into_iter()
        .map(From::from)
        .collect(),
        allowed_headers: rocket_cors::AllowedHeaders::all(),
        allow_credentials: true,
        ..Default::default()
    }
    .to_cors()
    .unwrap();

    rocket::build()
        .attach(cors)
        .manage(hub)
        .mount(
            "/api",
            routes![
                auth_handler::register_handler,
                auth_handler::login_handler,
                auth_handler::me_handler,
                auth_handler::logout_handler,
                ws_handler::ws_handler,
                resume_handler::list_resumes_handler,
                resume_handler::list_resume_handler,
                resume_handler::create_resume_handler,
                resume_handler::update_resume_handler,
                resume_handler::delete_resume_handler,
                skills_handler::list_skills_handler,
                skills_handler::create_skill_handler,
                skills_handler::update_skill_handler,
                skills_handler::delete_skill_handler,
                languages_handler::list_languages_handler,
                languages_handler::create_language_handler,
                languages_handler::update_language_handler,
                languages_handler::delete_language_handler,
                frameworks_handler::list_frameworks_handler,
                frameworks_handler::create_framework_handler,
                frameworks_handler::update_framework_handler,
                frameworks_handler::delete_framework_handler,
                education_handler::list_educations_handler,
                education_handler::create_education_handler,
                education_handler::update_education_handler,
                education_handler::delete_education_handler,
                education_handler::list_education_key_points_handler,
                education_handler::create_education_key_point_handler,
                education_handler::update_education_key_point_handler,
                education_handler::delete_education_key_point_handler,
                work_experiences_handler::list_work_experiences_handler,
                work_experiences_handler::create_work_experience_handler,
                work_experiences_handler::update_work_experience_handler,
                work_experiences_handler::delete_work_experience_handler,
                work_experiences_handler::list_work_experience_key_points_handler,
                work_experiences_handler::create_work_experience_key_point_handler,
                work_experiences_handler::update_work_experience_key_point_handler,
                work_experiences_handler::delete_work_experience_key_point_handler,
                portfolio_projects_handler::list_portfolio_projects_handler,
                portfolio_projects_handler::create_portfolio_project_handler,
                portfolio_projects_handler::update_portfolio_project_handler,
                portfolio_projects_handler::delete_portfolio_project_handler,
                portfolio_projects_handler::list_portfolio_key_points_handler,
                portfolio_projects_handler::create_portfolio_key_point_handler,
                portfolio_projects_handler::update_portfolio_key_point_handler,
                portfolio_projects_handler::delete_portfolio_key_point_handler,
                portfolio_projects_handler::list_portfolio_technologies_handler,
                portfolio_projects_handler::create_portfolio_technology_handler,
                portfolio_projects_handler::update_portfolio_technology_handler,
                portfolio_projects_handler::delete_portfolio_technology_handler,
            ],
        )
        .mount(
            "/",
            utoipa_swagger_ui::SwaggerUi::new("/api/docs/<_..>")
                .url("/api/openapi.json", openapi::ApiDoc::openapi()),
        )
}
