use application::error::ApplicationError;
use application::resume::portfolio_projects;
use domain::models::{
    NewPortfolioKeyPointRequest, NewPortfolioProjectRequest, NewPortfolioTechnologyRequest,
    UpdatePortfolioKeyPoint, UpdatePortfolioProject, UpdatePortfolioTechnology,
};
use rocket::response::status::{Custom, NoContent};
use rocket::serde::json::Json;
use rocket::{delete as rocket_delete, get, post, put};
use shared::response_models::Response;

use crate::auth::{AuthSession, MaybeAuthSession};

#[get("/resume/<resume_id>/portfolio_projects")]
pub fn list_portfolio_projects_handler(
    resume_id: i32,
    maybe_auth: MaybeAuthSession,
) -> Result<Json<Response<Vec<domain::models::PortfolioProject>>>, Custom<Json<Response<String>>>> {
    let user_id_value = maybe_auth.0.map(|a| a.user_id);

    match portfolio_projects::list_portfolio_projects(resume_id, user_id_value) {
        Ok(items) => Ok(Json(Response { body: items })),
        Err(ApplicationError::NotFound(msg)) => Err(Custom(
            rocket::http::Status::NotFound,
            Json(Response { body: msg }),
        )),
        Err(ApplicationError::Forbidden) => Err(Custom(
            rocket::http::Status::Forbidden,
            Json(Response {
                body: "Forbidden".to_string(),
            }),
        )),
        Err(ApplicationError::Unauthorized) => Err(Custom(
            rocket::http::Status::Unauthorized,
            Json(Response {
                body: "Unauthorized".to_string(),
            }),
        )),
        Err(ApplicationError::Conflict(msg)) => Err(Custom(
            rocket::http::Status::Conflict,
            Json(Response { body: msg }),
        )),
        Err(ApplicationError::BadRequest(msg)) => Err(Custom(
            rocket::http::Status::BadRequest,
            Json(Response { body: msg }),
        )),
        Err(ApplicationError::Internal(msg)) => Err(Custom(
            rocket::http::Status::InternalServerError,
            Json(Response { body: msg }),
        )),
    }
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
) -> Result<Custom<Json<Response<domain::models::PortfolioProject>>>, Custom<Json<Response<String>>>>
{
    match portfolio_projects::create_portfolio_project(
        auth.user_id,
        resume_id,
        payload.into_inner(),
    ) {
        Ok(item) => Ok(Custom(
            rocket::http::Status::Created,
            Json(Response { body: item }),
        )),
        Err(ApplicationError::NotFound(msg)) => Err(Custom(
            rocket::http::Status::NotFound,
            Json(Response { body: msg }),
        )),
        Err(ApplicationError::Forbidden) => Err(Custom(
            rocket::http::Status::Forbidden,
            Json(Response {
                body: "Forbidden".to_string(),
            }),
        )),
        Err(ApplicationError::Unauthorized) => Err(Custom(
            rocket::http::Status::Unauthorized,
            Json(Response {
                body: "Unauthorized".to_string(),
            }),
        )),
        Err(ApplicationError::Conflict(msg)) => Err(Custom(
            rocket::http::Status::Conflict,
            Json(Response { body: msg }),
        )),
        Err(ApplicationError::BadRequest(msg)) => Err(Custom(
            rocket::http::Status::BadRequest,
            Json(Response { body: msg }),
        )),
        Err(ApplicationError::Internal(msg)) => Err(Custom(
            rocket::http::Status::InternalServerError,
            Json(Response { body: msg }),
        )),
    }
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
) -> Result<Json<Response<domain::models::PortfolioProject>>, Custom<Json<Response<String>>>> {
    match portfolio_projects::update_portfolio_project(
        auth.user_id,
        project_id,
        payload.into_inner(),
    ) {
        Ok(item) => Ok(Json(Response { body: item })),
        Err(ApplicationError::NotFound(msg)) => Err(Custom(
            rocket::http::Status::NotFound,
            Json(Response { body: msg }),
        )),
        Err(ApplicationError::Forbidden) => Err(Custom(
            rocket::http::Status::Forbidden,
            Json(Response {
                body: "Forbidden".to_string(),
            }),
        )),
        Err(ApplicationError::Unauthorized) => Err(Custom(
            rocket::http::Status::Unauthorized,
            Json(Response {
                body: "Unauthorized".to_string(),
            }),
        )),
        Err(ApplicationError::Conflict(msg)) => Err(Custom(
            rocket::http::Status::Conflict,
            Json(Response { body: msg }),
        )),
        Err(ApplicationError::BadRequest(msg)) => Err(Custom(
            rocket::http::Status::BadRequest,
            Json(Response { body: msg }),
        )),
        Err(ApplicationError::Internal(msg)) => Err(Custom(
            rocket::http::Status::InternalServerError,
            Json(Response { body: msg }),
        )),
    }
}

#[rocket_delete("/portfolio_projects/<project_id>")]
pub fn delete_portfolio_project_handler(
    auth: AuthSession,
    project_id: i32,
) -> Result<NoContent, Custom<Json<Response<String>>>> {
    match portfolio_projects::delete_portfolio_project(auth.user_id, project_id) {
        Ok(()) => Ok(NoContent),
        Err(ApplicationError::NotFound(msg)) => Err(Custom(
            rocket::http::Status::NotFound,
            Json(Response { body: msg }),
        )),
        Err(ApplicationError::Forbidden) => Err(Custom(
            rocket::http::Status::Forbidden,
            Json(Response {
                body: "Forbidden".to_string(),
            }),
        )),
        Err(ApplicationError::Unauthorized) => Err(Custom(
            rocket::http::Status::Unauthorized,
            Json(Response {
                body: "Unauthorized".to_string(),
            }),
        )),
        Err(ApplicationError::Conflict(msg)) => Err(Custom(
            rocket::http::Status::Conflict,
            Json(Response { body: msg }),
        )),
        Err(ApplicationError::BadRequest(msg)) => Err(Custom(
            rocket::http::Status::BadRequest,
            Json(Response { body: msg }),
        )),
        Err(ApplicationError::Internal(msg)) => Err(Custom(
            rocket::http::Status::InternalServerError,
            Json(Response { body: msg }),
        )),
    }
}

#[get("/resume/<resume_id>/portfolio_projects/<project_id>/key_points")]
pub fn list_portfolio_key_points_handler(
    resume_id: i32,
    project_id: i32,
    maybe_auth: MaybeAuthSession,
) -> Result<Json<Response<Vec<domain::models::PortfolioKeyPoint>>>, Custom<Json<Response<String>>>>
{
    let user_id_value = maybe_auth.0.map(|a| a.user_id);

    match portfolio_projects::list_portfolio_key_points(resume_id, project_id, user_id_value) {
        Ok(items) => Ok(Json(Response { body: items })),
        Err(ApplicationError::NotFound(msg)) => Err(Custom(
            rocket::http::Status::NotFound,
            Json(Response { body: msg }),
        )),
        Err(ApplicationError::Forbidden) => Err(Custom(
            rocket::http::Status::Forbidden,
            Json(Response {
                body: "Forbidden".to_string(),
            }),
        )),
        Err(ApplicationError::Unauthorized) => Err(Custom(
            rocket::http::Status::Unauthorized,
            Json(Response {
                body: "Unauthorized".to_string(),
            }),
        )),
        Err(ApplicationError::Conflict(msg)) => Err(Custom(
            rocket::http::Status::Conflict,
            Json(Response { body: msg }),
        )),
        Err(ApplicationError::BadRequest(msg)) => Err(Custom(
            rocket::http::Status::BadRequest,
            Json(Response { body: msg }),
        )),
        Err(ApplicationError::Internal(msg)) => Err(Custom(
            rocket::http::Status::InternalServerError,
            Json(Response { body: msg }),
        )),
    }
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
) -> Result<Custom<Json<Response<domain::models::PortfolioKeyPoint>>>, Custom<Json<Response<String>>>>
{
    match portfolio_projects::create_portfolio_key_point(
        auth.user_id,
        resume_id,
        project_id,
        payload.into_inner(),
    ) {
        Ok(item) => Ok(Custom(
            rocket::http::Status::Created,
            Json(Response { body: item }),
        )),
        Err(ApplicationError::NotFound(msg)) => Err(Custom(
            rocket::http::Status::NotFound,
            Json(Response { body: msg }),
        )),
        Err(ApplicationError::Forbidden) => Err(Custom(
            rocket::http::Status::Forbidden,
            Json(Response {
                body: "Forbidden".to_string(),
            }),
        )),
        Err(ApplicationError::Unauthorized) => Err(Custom(
            rocket::http::Status::Unauthorized,
            Json(Response {
                body: "Unauthorized".to_string(),
            }),
        )),
        Err(ApplicationError::Conflict(msg)) => Err(Custom(
            rocket::http::Status::Conflict,
            Json(Response { body: msg }),
        )),
        Err(ApplicationError::BadRequest(msg)) => Err(Custom(
            rocket::http::Status::BadRequest,
            Json(Response { body: msg }),
        )),
        Err(ApplicationError::Internal(msg)) => Err(Custom(
            rocket::http::Status::InternalServerError,
            Json(Response { body: msg }),
        )),
    }
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
) -> Result<Json<Response<domain::models::PortfolioKeyPoint>>, Custom<Json<Response<String>>>> {
    match portfolio_projects::update_portfolio_key_point(
        auth.user_id,
        key_point_id,
        payload.into_inner(),
    ) {
        Ok(item) => Ok(Json(Response { body: item })),
        Err(ApplicationError::NotFound(msg)) => Err(Custom(
            rocket::http::Status::NotFound,
            Json(Response { body: msg }),
        )),
        Err(ApplicationError::Forbidden) => Err(Custom(
            rocket::http::Status::Forbidden,
            Json(Response {
                body: "Forbidden".to_string(),
            }),
        )),
        Err(ApplicationError::Unauthorized) => Err(Custom(
            rocket::http::Status::Unauthorized,
            Json(Response {
                body: "Unauthorized".to_string(),
            }),
        )),
        Err(ApplicationError::Conflict(msg)) => Err(Custom(
            rocket::http::Status::Conflict,
            Json(Response { body: msg }),
        )),
        Err(ApplicationError::BadRequest(msg)) => Err(Custom(
            rocket::http::Status::BadRequest,
            Json(Response { body: msg }),
        )),
        Err(ApplicationError::Internal(msg)) => Err(Custom(
            rocket::http::Status::InternalServerError,
            Json(Response { body: msg }),
        )),
    }
}

#[rocket_delete("/portfolio_key_points/<key_point_id>")]
pub fn delete_portfolio_key_point_handler(
    auth: AuthSession,
    key_point_id: i32,
) -> Result<NoContent, Custom<Json<Response<String>>>> {
    match portfolio_projects::delete_portfolio_key_point(auth.user_id, key_point_id) {
        Ok(()) => Ok(NoContent),
        Err(ApplicationError::NotFound(msg)) => Err(Custom(
            rocket::http::Status::NotFound,
            Json(Response { body: msg }),
        )),
        Err(ApplicationError::Forbidden) => Err(Custom(
            rocket::http::Status::Forbidden,
            Json(Response {
                body: "Forbidden".to_string(),
            }),
        )),
        Err(ApplicationError::Unauthorized) => Err(Custom(
            rocket::http::Status::Unauthorized,
            Json(Response {
                body: "Unauthorized".to_string(),
            }),
        )),
        Err(ApplicationError::Conflict(msg)) => Err(Custom(
            rocket::http::Status::Conflict,
            Json(Response { body: msg }),
        )),
        Err(ApplicationError::BadRequest(msg)) => Err(Custom(
            rocket::http::Status::BadRequest,
            Json(Response { body: msg }),
        )),
        Err(ApplicationError::Internal(msg)) => Err(Custom(
            rocket::http::Status::InternalServerError,
            Json(Response { body: msg }),
        )),
    }
}

#[get("/resume/<resume_id>/portfolio_projects/<project_id>/technologies")]
pub fn list_portfolio_technologies_handler(
    resume_id: i32,
    project_id: i32,
    maybe_auth: MaybeAuthSession,
) -> Result<Json<Response<Vec<domain::models::PortfolioTechnology>>>, Custom<Json<Response<String>>>>
{
    let user_id_value = maybe_auth.0.map(|a| a.user_id);

    match portfolio_projects::list_portfolio_technologies(resume_id, project_id, user_id_value) {
        Ok(items) => Ok(Json(Response { body: items })),
        Err(ApplicationError::NotFound(msg)) => Err(Custom(
            rocket::http::Status::NotFound,
            Json(Response { body: msg }),
        )),
        Err(ApplicationError::Forbidden) => Err(Custom(
            rocket::http::Status::Forbidden,
            Json(Response {
                body: "Forbidden".to_string(),
            }),
        )),
        Err(ApplicationError::Unauthorized) => Err(Custom(
            rocket::http::Status::Unauthorized,
            Json(Response {
                body: "Unauthorized".to_string(),
            }),
        )),
        Err(ApplicationError::Conflict(msg)) => Err(Custom(
            rocket::http::Status::Conflict,
            Json(Response { body: msg }),
        )),
        Err(ApplicationError::BadRequest(msg)) => Err(Custom(
            rocket::http::Status::BadRequest,
            Json(Response { body: msg }),
        )),
        Err(ApplicationError::Internal(msg)) => Err(Custom(
            rocket::http::Status::InternalServerError,
            Json(Response { body: msg }),
        )),
    }
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
) -> Result<
    Custom<Json<Response<domain::models::PortfolioTechnology>>>,
    Custom<Json<Response<String>>>,
> {
    match portfolio_projects::create_portfolio_technology(
        auth.user_id,
        resume_id,
        project_id,
        payload.into_inner(),
    ) {
        Ok(item) => Ok(Custom(
            rocket::http::Status::Created,
            Json(Response { body: item }),
        )),
        Err(ApplicationError::NotFound(msg)) => Err(Custom(
            rocket::http::Status::NotFound,
            Json(Response { body: msg }),
        )),
        Err(ApplicationError::Forbidden) => Err(Custom(
            rocket::http::Status::Forbidden,
            Json(Response {
                body: "Forbidden".to_string(),
            }),
        )),
        Err(ApplicationError::Unauthorized) => Err(Custom(
            rocket::http::Status::Unauthorized,
            Json(Response {
                body: "Unauthorized".to_string(),
            }),
        )),
        Err(ApplicationError::Conflict(msg)) => Err(Custom(
            rocket::http::Status::Conflict,
            Json(Response { body: msg }),
        )),
        Err(ApplicationError::BadRequest(msg)) => Err(Custom(
            rocket::http::Status::BadRequest,
            Json(Response { body: msg }),
        )),
        Err(ApplicationError::Internal(msg)) => Err(Custom(
            rocket::http::Status::InternalServerError,
            Json(Response { body: msg }),
        )),
    }
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
) -> Result<Json<Response<domain::models::PortfolioTechnology>>, Custom<Json<Response<String>>>> {
    match portfolio_projects::update_portfolio_technology(
        auth.user_id,
        technology_id,
        payload.into_inner(),
    ) {
        Ok(item) => Ok(Json(Response { body: item })),
        Err(ApplicationError::NotFound(msg)) => Err(Custom(
            rocket::http::Status::NotFound,
            Json(Response { body: msg }),
        )),
        Err(ApplicationError::Forbidden) => Err(Custom(
            rocket::http::Status::Forbidden,
            Json(Response {
                body: "Forbidden".to_string(),
            }),
        )),
        Err(ApplicationError::Unauthorized) => Err(Custom(
            rocket::http::Status::Unauthorized,
            Json(Response {
                body: "Unauthorized".to_string(),
            }),
        )),
        Err(ApplicationError::Conflict(msg)) => Err(Custom(
            rocket::http::Status::Conflict,
            Json(Response { body: msg }),
        )),
        Err(ApplicationError::BadRequest(msg)) => Err(Custom(
            rocket::http::Status::BadRequest,
            Json(Response { body: msg }),
        )),
        Err(ApplicationError::Internal(msg)) => Err(Custom(
            rocket::http::Status::InternalServerError,
            Json(Response { body: msg }),
        )),
    }
}

#[rocket_delete("/portfolio_technologies/<technology_id>")]
pub fn delete_portfolio_technology_handler(
    auth: AuthSession,
    technology_id: i32,
) -> Result<NoContent, Custom<Json<Response<String>>>> {
    match portfolio_projects::delete_portfolio_technology(auth.user_id, technology_id) {
        Ok(()) => Ok(NoContent),
        Err(ApplicationError::NotFound(msg)) => Err(Custom(
            rocket::http::Status::NotFound,
            Json(Response { body: msg }),
        )),
        Err(ApplicationError::Forbidden) => Err(Custom(
            rocket::http::Status::Forbidden,
            Json(Response {
                body: "Forbidden".to_string(),
            }),
        )),
        Err(ApplicationError::Unauthorized) => Err(Custom(
            rocket::http::Status::Unauthorized,
            Json(Response {
                body: "Unauthorized".to_string(),
            }),
        )),
        Err(ApplicationError::Conflict(msg)) => Err(Custom(
            rocket::http::Status::Conflict,
            Json(Response { body: msg }),
        )),
        Err(ApplicationError::BadRequest(msg)) => Err(Custom(
            rocket::http::Status::BadRequest,
            Json(Response { body: msg }),
        )),
        Err(ApplicationError::Internal(msg)) => Err(Custom(
            rocket::http::Status::InternalServerError,
            Json(Response { body: msg }),
        )),
    }
}
