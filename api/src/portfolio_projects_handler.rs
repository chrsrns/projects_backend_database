use application::error::ApplicationError;
use application::resume::portfolio_projects;
use domain::models::{
    NewPortfolioKeyPointRequest, NewPortfolioProjectRequest, NewPortfolioTechnologyRequest,
    PortfolioKeyPoint, PortfolioProject, PortfolioTechnology, UpdatePortfolioKeyPoint,
    UpdatePortfolioProject, UpdatePortfolioTechnology,
};
use rocket::response::status::{Custom, NoContent};
use rocket::serde::json::Json;
use rocket::{delete as rocket_delete, get, post, put};
use shared::response_models::Response;

use crate::auth::{AuthSession, MaybeAuthSession};

#[utoipa::path(
    get,
    path = "/resume/{resume_id}/portfolio_projects",
    tag = "PortfolioProjects",
    params(
        ("resume_id" = i32, Path, description = "Resume id")
    ),
    responses(
        (status = 200, description = "OK", body = Response<Vec<PortfolioProject>>, content_type = "application/json"),
        (status = 401, description = "Unauthorized", body = Response<String>, content_type = "application/json"),
        (status = 403, description = "Forbidden", body = Response<String>, content_type = "application/json"),
        (status = 404, description = "Not Found", body = Response<String>, content_type = "application/json")
    )
)]
#[get("/resume/<resume_id>/portfolio_projects")]
pub fn list_portfolio_projects_handler(
    resume_id: i32,
    maybe_auth: MaybeAuthSession,
) -> Result<Json<Response<Vec<PortfolioProject>>>, Custom<Json<Response<String>>>> {
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

#[utoipa::path(
    post,
    path = "/resume/{resume_id}/portfolio_projects",
    tag = "PortfolioProjects",
    security(("bearerAuth" = [])),
    params(
        ("resume_id" = i32, Path, description = "Resume id")
    ),
    request_body(content = NewPortfolioProjectRequest, content_type = "application/json"),
    responses(
        (status = 201, description = "Created", body = Response<PortfolioProject>, content_type = "application/json"),
        (status = 401, description = "Unauthorized", body = Response<String>, content_type = "application/json"),
        (status = 403, description = "Forbidden", body = Response<String>, content_type = "application/json"),
        (status = 404, description = "Not Found", body = Response<String>, content_type = "application/json")
    )
)]
#[post(
    "/resume/<resume_id>/portfolio_projects",
    format = "application/json",
    data = "<payload>"
)]
pub fn create_portfolio_project_handler(
    auth: AuthSession,
    resume_id: i32,
    payload: Json<NewPortfolioProjectRequest>,
) -> Result<Custom<Json<Response<PortfolioProject>>>, Custom<Json<Response<String>>>> {
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

#[utoipa::path(
    put,
    path = "/portfolio_projects/{project_id}",
    tag = "PortfolioProjects",
    security(("bearerAuth" = [])),
    params(
        ("project_id" = i32, Path, description = "Portfolio project id")
    ),
    request_body(content = UpdatePortfolioProject, content_type = "application/json"),
    responses(
        (status = 200, description = "OK", body = Response<PortfolioProject>, content_type = "application/json"),
        (status = 401, description = "Unauthorized", body = Response<String>, content_type = "application/json"),
        (status = 403, description = "Forbidden", body = Response<String>, content_type = "application/json"),
        (status = 404, description = "Not Found", body = Response<String>, content_type = "application/json")
    )
)]
#[put(
    "/portfolio_projects/<project_id>",
    format = "application/json",
    data = "<payload>"
)]
pub fn update_portfolio_project_handler(
    auth: AuthSession,
    project_id: i32,
    payload: Json<UpdatePortfolioProject>,
) -> Result<Json<Response<PortfolioProject>>, Custom<Json<Response<String>>>> {
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

#[utoipa::path(
    delete,
    path = "/portfolio_projects/{project_id}",
    tag = "PortfolioProjects",
    security(("bearerAuth" = [])),
    params(
        ("project_id" = i32, Path, description = "Portfolio project id")
    ),
    responses(
        (status = 204, description = "No Content"),
        (status = 401, description = "Unauthorized", body = Response<String>, content_type = "application/json"),
        (status = 403, description = "Forbidden", body = Response<String>, content_type = "application/json"),
        (status = 404, description = "Not Found", body = Response<String>, content_type = "application/json")
    )
)]
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

#[utoipa::path(
    get,
    path = "/resume/{resume_id}/portfolio_projects/{project_id}/key_points",
    tag = "PortfolioKeyPoints",
    params(
        ("resume_id" = i32, Path, description = "Resume id"),
        ("project_id" = i32, Path, description = "Portfolio project id")
    ),
    responses(
        (status = 200, description = "OK", body = Response<Vec<PortfolioKeyPoint>>, content_type = "application/json"),
        (status = 401, description = "Unauthorized", body = Response<String>, content_type = "application/json"),
        (status = 403, description = "Forbidden", body = Response<String>, content_type = "application/json"),
        (status = 404, description = "Not Found", body = Response<String>, content_type = "application/json")
    )
)]
#[get("/resume/<resume_id>/portfolio_projects/<project_id>/key_points")]
pub fn list_portfolio_key_points_handler(
    resume_id: i32,
    project_id: i32,
    maybe_auth: MaybeAuthSession,
) -> Result<Json<Response<Vec<PortfolioKeyPoint>>>, Custom<Json<Response<String>>>> {
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

#[utoipa::path(
    post,
    path = "/resume/{resume_id}/portfolio_projects/{project_id}/key_points",
    tag = "PortfolioKeyPoints",
    security(("bearerAuth" = [])),
    params(
        ("resume_id" = i32, Path, description = "Resume id"),
        ("project_id" = i32, Path, description = "Portfolio project id")
    ),
    request_body(content = NewPortfolioKeyPointRequest, content_type = "application/json"),
    responses(
        (status = 201, description = "Created", body = Response<PortfolioKeyPoint>, content_type = "application/json"),
        (status = 401, description = "Unauthorized", body = Response<String>, content_type = "application/json"),
        (status = 403, description = "Forbidden", body = Response<String>, content_type = "application/json"),
        (status = 404, description = "Not Found", body = Response<String>, content_type = "application/json")
    )
)]
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
) -> Result<Custom<Json<Response<PortfolioKeyPoint>>>, Custom<Json<Response<String>>>> {
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

#[utoipa::path(
    put,
    path = "/portfolio_key_points/{key_point_id}",
    tag = "PortfolioKeyPoints",
    security(("bearerAuth" = [])),
    params(
        ("key_point_id" = i32, Path, description = "Key point id")
    ),
    request_body(content = UpdatePortfolioKeyPoint, content_type = "application/json"),
    responses(
        (status = 200, description = "OK", body = Response<PortfolioKeyPoint>, content_type = "application/json"),
        (status = 401, description = "Unauthorized", body = Response<String>, content_type = "application/json"),
        (status = 403, description = "Forbidden", body = Response<String>, content_type = "application/json"),
        (status = 404, description = "Not Found", body = Response<String>, content_type = "application/json")
    )
)]
#[put(
    "/portfolio_key_points/<key_point_id>",
    format = "application/json",
    data = "<payload>"
)]
pub fn update_portfolio_key_point_handler(
    auth: AuthSession,
    key_point_id: i32,
    payload: Json<UpdatePortfolioKeyPoint>,
) -> Result<Json<Response<PortfolioKeyPoint>>, Custom<Json<Response<String>>>> {
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

#[utoipa::path(
    delete,
    path = "/portfolio_key_points/{key_point_id}",
    tag = "PortfolioKeyPoints",
    security(("bearerAuth" = [])),
    params(
        ("key_point_id" = i32, Path, description = "Key point id")
    ),
    responses(
        (status = 204, description = "No Content"),
        (status = 401, description = "Unauthorized", body = Response<String>, content_type = "application/json"),
        (status = 403, description = "Forbidden", body = Response<String>, content_type = "application/json"),
        (status = 404, description = "Not Found", body = Response<String>, content_type = "application/json")
    )
)]
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

#[utoipa::path(
    get,
    path = "/resume/{resume_id}/portfolio_projects/{project_id}/technologies",
    tag = "PortfolioTechnologies",
    params(
        ("resume_id" = i32, Path, description = "Resume id"),
        ("project_id" = i32, Path, description = "Portfolio project id")
    ),
    responses(
        (status = 200, description = "OK", body = Response<Vec<PortfolioTechnology>>, content_type = "application/json"),
        (status = 401, description = "Unauthorized", body = Response<String>, content_type = "application/json"),
        (status = 403, description = "Forbidden", body = Response<String>, content_type = "application/json"),
        (status = 404, description = "Not Found", body = Response<String>, content_type = "application/json")
    )
)]
#[get("/resume/<resume_id>/portfolio_projects/<project_id>/technologies")]
pub fn list_portfolio_technologies_handler(
    resume_id: i32,
    project_id: i32,
    maybe_auth: MaybeAuthSession,
) -> Result<Json<Response<Vec<PortfolioTechnology>>>, Custom<Json<Response<String>>>> {
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

#[utoipa::path(
    post,
    path = "/resume/{resume_id}/portfolio_projects/{project_id}/technologies",
    tag = "PortfolioTechnologies",
    security(("bearerAuth" = [])),
    params(
        ("resume_id" = i32, Path, description = "Resume id"),
        ("project_id" = i32, Path, description = "Portfolio project id")
    ),
    request_body(content = NewPortfolioTechnologyRequest, content_type = "application/json"),
    responses(
        (status = 201, description = "Created", body = Response<PortfolioTechnology>, content_type = "application/json"),
        (status = 401, description = "Unauthorized", body = Response<String>, content_type = "application/json"),
        (status = 403, description = "Forbidden", body = Response<String>, content_type = "application/json"),
        (status = 404, description = "Not Found", body = Response<String>, content_type = "application/json")
    )
)]
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
) -> Result<Custom<Json<Response<PortfolioTechnology>>>, Custom<Json<Response<String>>>> {
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

#[utoipa::path(
    put,
    path = "/portfolio_technologies/{technology_id}",
    tag = "PortfolioTechnologies",
    security(("bearerAuth" = [])),
    params(
        ("technology_id" = i32, Path, description = "Technology id")
    ),
    request_body(content = UpdatePortfolioTechnology, content_type = "application/json"),
    responses(
        (status = 200, description = "OK", body = Response<PortfolioTechnology>, content_type = "application/json"),
        (status = 401, description = "Unauthorized", body = Response<String>, content_type = "application/json"),
        (status = 403, description = "Forbidden", body = Response<String>, content_type = "application/json"),
        (status = 404, description = "Not Found", body = Response<String>, content_type = "application/json")
    )
)]
#[put(
    "/portfolio_technologies/<technology_id>",
    format = "application/json",
    data = "<payload>"
)]
pub fn update_portfolio_technology_handler(
    auth: AuthSession,
    technology_id: i32,
    payload: Json<UpdatePortfolioTechnology>,
) -> Result<Json<Response<PortfolioTechnology>>, Custom<Json<Response<String>>>> {
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

#[utoipa::path(
    delete,
    path = "/portfolio_technologies/{technology_id}",
    tag = "PortfolioTechnologies",
    security(("bearerAuth" = [])),
    params(
        ("technology_id" = i32, Path, description = "Technology id")
    ),
    responses(
        (status = 204, description = "No Content"),
        (status = 401, description = "Unauthorized", body = Response<String>, content_type = "application/json"),
        (status = 403, description = "Forbidden", body = Response<String>, content_type = "application/json"),
        (status = 404, description = "Not Found", body = Response<String>, content_type = "application/json")
    )
)]
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
