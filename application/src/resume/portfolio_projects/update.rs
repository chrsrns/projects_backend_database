use diesel::prelude::*;
use domain::models::{
    PortfolioKeyPoint, PortfolioProject, PortfolioTechnology, Resume, UpdatePortfolioKeyPoint,
    UpdatePortfolioProject, UpdatePortfolioTechnology,
};
use infrastructure::establish_connection;
use rocket::http::Status;
use rocket::response::status::Custom;
use rocket::serde::json::Json;
use shared::response_models::{Response, ResponseBody};

pub fn update_portfolio_project(
    user_id_value: i32,
    project_id_value: i32,
    payload: Json<UpdatePortfolioProject>,
) -> Result<String, Custom<String>> {
    use domain::schema::portfolio_projects;
    use domain::schema::resumes;

    let existing: PortfolioProject = match portfolio_projects::table
        .find(project_id_value)
        .first(&mut establish_connection())
    {
        Ok(v) => v,
        Err(diesel::result::Error::NotFound) => {
            let response = Response {
                body: ResponseBody::Message(format!(
                    "Portfolio project with id {} not found",
                    project_id_value
                )),
            };
            return Err(Custom(
                Status::NotFound,
                serde_json::to_string(&response).unwrap(),
            ));
        }
        Err(err) => panic!("Database error - {}", err),
    };

    let resume: Resume = match resumes::table
        .find(existing.resume_id)
        .first(&mut establish_connection())
    {
        Ok(r) => r,
        Err(diesel::result::Error::NotFound) => {
            let response = Response {
                body: ResponseBody::Message("Resume not found".to_string()),
            };
            return Err(Custom(
                Status::NotFound,
                serde_json::to_string(&response).unwrap(),
            ));
        }
        Err(err) => panic!("Database error - {}", err),
    };

    match resume.created_by {
        Some(owner) if owner == user_id_value => {}
        Some(_) | None => {
            let response = Response {
                body: ResponseBody::Message("Forbidden".to_string()),
            };
            return Err(Custom(
                Status::Forbidden,
                serde_json::to_string(&response).unwrap(),
            ));
        }
    }

    let payload = payload.into_inner();
    match diesel::update(portfolio_projects::table.find(project_id_value))
        .set(&payload)
        .get_result::<PortfolioProject>(&mut establish_connection())
    {
        Ok(updated) => {
            let response = Response {
                body: ResponseBody::PortfolioProject(updated),
            };
            Ok(serde_json::to_string(&response).unwrap())
        }
        Err(diesel::result::Error::NotFound) => {
            let response = Response {
                body: ResponseBody::Message(format!(
                    "Portfolio project with id {} not found",
                    project_id_value
                )),
            };
            Err(Custom(
                Status::NotFound,
                serde_json::to_string(&response).unwrap(),
            ))
        }
        Err(err) => panic!("Database error - {}", err),
    }
}

pub fn update_portfolio_key_point(
    user_id_value: i32,
    key_point_id_value: i32,
    payload: Json<UpdatePortfolioKeyPoint>,
) -> Result<String, Custom<String>> {
    use domain::schema::portfolio_key_points;
    use domain::schema::portfolio_projects;
    use domain::schema::resumes;

    let existing: PortfolioKeyPoint = match portfolio_key_points::table
        .find(key_point_id_value)
        .first(&mut establish_connection())
    {
        Ok(v) => v,
        Err(diesel::result::Error::NotFound) => {
            let response = Response {
                body: ResponseBody::Message(format!(
                    "Portfolio key point with id {} not found",
                    key_point_id_value
                )),
            };
            return Err(Custom(
                Status::NotFound,
                serde_json::to_string(&response).unwrap(),
            ));
        }
        Err(err) => panic!("Database error - {}", err),
    };

    let project: PortfolioProject = match portfolio_projects::table
        .find(existing.portfolio_project_id)
        .first(&mut establish_connection())
    {
        Ok(p) => p,
        Err(diesel::result::Error::NotFound) => {
            let response = Response {
                body: ResponseBody::Message("Portfolio project not found".to_string()),
            };
            return Err(Custom(
                Status::NotFound,
                serde_json::to_string(&response).unwrap(),
            ));
        }
        Err(err) => panic!("Database error - {}", err),
    };

    let resume: Resume = match resumes::table
        .find(project.resume_id)
        .first(&mut establish_connection())
    {
        Ok(r) => r,
        Err(diesel::result::Error::NotFound) => {
            let response = Response {
                body: ResponseBody::Message("Resume not found".to_string()),
            };
            return Err(Custom(
                Status::NotFound,
                serde_json::to_string(&response).unwrap(),
            ));
        }
        Err(err) => panic!("Database error - {}", err),
    };

    match resume.created_by {
        Some(owner) if owner == user_id_value => {}
        Some(_) | None => {
            let response = Response {
                body: ResponseBody::Message("Forbidden".to_string()),
            };
            return Err(Custom(
                Status::Forbidden,
                serde_json::to_string(&response).unwrap(),
            ));
        }
    }

    let payload = payload.into_inner();
    match diesel::update(portfolio_key_points::table.find(key_point_id_value))
        .set(&payload)
        .get_result::<PortfolioKeyPoint>(&mut establish_connection())
    {
        Ok(updated) => {
            let response = Response {
                body: ResponseBody::PortfolioKeyPoint(updated),
            };
            Ok(serde_json::to_string(&response).unwrap())
        }
        Err(diesel::result::Error::NotFound) => {
            let response = Response {
                body: ResponseBody::Message(format!(
                    "Portfolio key point with id {} not found",
                    key_point_id_value
                )),
            };
            Err(Custom(
                Status::NotFound,
                serde_json::to_string(&response).unwrap(),
            ))
        }
        Err(err) => panic!("Database error - {}", err),
    }
}

pub fn update_portfolio_technology(
    user_id_value: i32,
    tech_id_value: i32,
    payload: Json<UpdatePortfolioTechnology>,
) -> Result<String, Custom<String>> {
    use domain::schema::portfolio_projects;
    use domain::schema::portfolio_technologies;
    use domain::schema::resumes;

    let existing: PortfolioTechnology = match portfolio_technologies::table
        .find(tech_id_value)
        .first(&mut establish_connection())
    {
        Ok(v) => v,
        Err(diesel::result::Error::NotFound) => {
            let response = Response {
                body: ResponseBody::Message(format!(
                    "Portfolio technology with id {} not found",
                    tech_id_value
                )),
            };
            return Err(Custom(
                Status::NotFound,
                serde_json::to_string(&response).unwrap(),
            ));
        }
        Err(err) => panic!("Database error - {}", err),
    };

    let project: PortfolioProject = match portfolio_projects::table
        .find(existing.portfolio_project_id)
        .first(&mut establish_connection())
    {
        Ok(p) => p,
        Err(diesel::result::Error::NotFound) => {
            let response = Response {
                body: ResponseBody::Message("Portfolio project not found".to_string()),
            };
            return Err(Custom(
                Status::NotFound,
                serde_json::to_string(&response).unwrap(),
            ));
        }
        Err(err) => panic!("Database error - {}", err),
    };

    let resume: Resume = match resumes::table
        .find(project.resume_id)
        .first(&mut establish_connection())
    {
        Ok(r) => r,
        Err(diesel::result::Error::NotFound) => {
            let response = Response {
                body: ResponseBody::Message("Resume not found".to_string()),
            };
            return Err(Custom(
                Status::NotFound,
                serde_json::to_string(&response).unwrap(),
            ));
        }
        Err(err) => panic!("Database error - {}", err),
    };

    match resume.created_by {
        Some(owner) if owner == user_id_value => {}
        Some(_) | None => {
            let response = Response {
                body: ResponseBody::Message("Forbidden".to_string()),
            };
            return Err(Custom(
                Status::Forbidden,
                serde_json::to_string(&response).unwrap(),
            ));
        }
    }

    let payload = payload.into_inner();
    match diesel::update(portfolio_technologies::table.find(tech_id_value))
        .set(&payload)
        .get_result::<PortfolioTechnology>(&mut establish_connection())
    {
        Ok(updated) => {
            let response = Response {
                body: ResponseBody::PortfolioTechnology(updated),
            };
            Ok(serde_json::to_string(&response).unwrap())
        }
        Err(diesel::result::Error::NotFound) => {
            let response = Response {
                body: ResponseBody::Message(format!(
                    "Portfolio technology with id {} not found",
                    tech_id_value
                )),
            };
            Err(Custom(
                Status::NotFound,
                serde_json::to_string(&response).unwrap(),
            ))
        }
        Err(err) => panic!("Database error - {}", err),
    }
}
