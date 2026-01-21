use application::resume::portfolio_projects;
use domain::models::{
    NewPortfolioKeyPointRequest, NewPortfolioProjectRequest, NewPortfolioTechnologyRequest,
    UpdatePortfolioKeyPoint, UpdatePortfolioProject, UpdatePortfolioTechnology,
};
use rocket::response::status::{Created, NoContent, NotFound};
use rocket::serde::json::Json;
use rocket::{delete as rocket_delete, get, post, put};

use crate::auth::{AuthSession, MaybeAuthSession};

#[get("/resume/<resume_id>/portfolio_projects")]
pub fn list_portfolio_projects_handler(
    resume_id: i32,
    maybe_auth: MaybeAuthSession,
) -> Result<String, NotFound<String>> {
    let user_id_value = maybe_auth.0.map(|a| a.user_id);
    portfolio_projects::list_portfolio_projects(resume_id, user_id_value)
}

#[post(
    "/resume/<resume_id>/portfolio_projects",
    format = "application/json",
    data = "<payload>"
)]
pub fn create_portfolio_project_handler(
    auth: AuthSession,
    resume_id: i32,
    payload: Json<NewPortfolioProjectRequest>,
) -> Result<Created<String>, rocket::response::status::Custom<String>> {
    portfolio_projects::create_portfolio_project(auth.user_id, resume_id, payload)
}

#[put(
    "/portfolio_projects/<project_id>",
    format = "application/json",
    data = "<payload>"
)]
pub fn update_portfolio_project_handler(
    auth: AuthSession,
    project_id: i32,
    payload: Json<UpdatePortfolioProject>,
) -> Result<String, rocket::response::status::Custom<String>> {
    portfolio_projects::update_portfolio_project(auth.user_id, project_id, payload)
}

#[rocket_delete("/portfolio_projects/<project_id>")]
pub fn delete_portfolio_project_handler(
    auth: AuthSession,
    project_id: i32,
) -> Result<NoContent, rocket::response::status::Custom<String>> {
    portfolio_projects::delete_portfolio_project(auth.user_id, project_id)
}

#[get("/resume/<resume_id>/portfolio_projects/<project_id>/key_points")]
pub fn list_portfolio_key_points_handler(
    resume_id: i32,
    project_id: i32,
    maybe_auth: MaybeAuthSession,
) -> Result<String, NotFound<String>> {
    let user_id_value = maybe_auth.0.map(|a| a.user_id);
    portfolio_projects::list_portfolio_key_points(resume_id, project_id, user_id_value)
}

#[post(
    "/resume/<resume_id>/portfolio_projects/<project_id>/key_points",
    format = "application/json",
    data = "<payload>"
)]
pub fn create_portfolio_key_point_handler(
    auth: AuthSession,
    resume_id: i32,
    project_id: i32,
    payload: Json<NewPortfolioKeyPointRequest>,
) -> Result<Created<String>, rocket::response::status::Custom<String>> {
    portfolio_projects::create_portfolio_key_point(auth.user_id, resume_id, project_id, payload)
}

#[put(
    "/portfolio_key_points/<key_point_id>",
    format = "application/json",
    data = "<payload>"
)]
pub fn update_portfolio_key_point_handler(
    auth: AuthSession,
    key_point_id: i32,
    payload: Json<UpdatePortfolioKeyPoint>,
) -> Result<String, rocket::response::status::Custom<String>> {
    portfolio_projects::update_portfolio_key_point(auth.user_id, key_point_id, payload)
}

#[rocket_delete("/portfolio_key_points/<key_point_id>")]
pub fn delete_portfolio_key_point_handler(
    auth: AuthSession,
    key_point_id: i32,
) -> Result<NoContent, rocket::response::status::Custom<String>> {
    portfolio_projects::delete_portfolio_key_point(auth.user_id, key_point_id)
}

#[get("/resume/<resume_id>/portfolio_projects/<project_id>/technologies")]
pub fn list_portfolio_technologies_handler(
    resume_id: i32,
    project_id: i32,
    maybe_auth: MaybeAuthSession,
) -> Result<String, NotFound<String>> {
    let user_id_value = maybe_auth.0.map(|a| a.user_id);
    portfolio_projects::list_portfolio_technologies(resume_id, project_id, user_id_value)
}

#[post(
    "/resume/<resume_id>/portfolio_projects/<project_id>/technologies",
    format = "application/json",
    data = "<payload>"
)]
pub fn create_portfolio_technology_handler(
    auth: AuthSession,
    resume_id: i32,
    project_id: i32,
    payload: Json<NewPortfolioTechnologyRequest>,
) -> Result<Created<String>, rocket::response::status::Custom<String>> {
    portfolio_projects::create_portfolio_technology(auth.user_id, resume_id, project_id, payload)
}

#[put(
    "/portfolio_technologies/<technology_id>",
    format = "application/json",
    data = "<payload>"
)]
pub fn update_portfolio_technology_handler(
    auth: AuthSession,
    technology_id: i32,
    payload: Json<UpdatePortfolioTechnology>,
) -> Result<String, rocket::response::status::Custom<String>> {
    portfolio_projects::update_portfolio_technology(auth.user_id, technology_id, payload)
}

#[rocket_delete("/portfolio_technologies/<technology_id>")]
pub fn delete_portfolio_technology_handler(
    auth: AuthSession,
    technology_id: i32,
) -> Result<NoContent, rocket::response::status::Custom<String>> {
    portfolio_projects::delete_portfolio_technology(auth.user_id, technology_id)
}
