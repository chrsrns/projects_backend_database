use diesel::prelude::*;
use domain::models::{PortfolioKeyPoint, PortfolioProject, PortfolioTechnology, Resume};
use infrastructure::establish_connection;

use crate::{
    error::ApplicationError,
    resume::common::{app_err_from_diesel_err, find_resume},
};

pub fn delete_portfolio_project(
    user_id_value: i32,
    project_id_value: i32,
) -> Result<i32, ApplicationError> {
    use domain::schema::portfolio_projects;

    let existing: PortfolioProject = match portfolio_projects::table
        .find(project_id_value)
        .first(&mut establish_connection())
    {
        Ok(v) => v,
        Err(err) => return Err(app_err_from_diesel_err(err)),
    };

    let resume: Resume = match find_resume(existing.resume_id) {
        Ok(r) => r,
        Err(err) => return Err(err),
    };

    match resume.created_by {
        Some(owner) if owner == user_id_value => {}
        Some(_) | None => {
            return Err(ApplicationError::Forbidden);
        }
    }

    match diesel::delete(portfolio_projects::table.find(project_id_value))
        .execute(&mut establish_connection())
    {
        Ok(count) => {
            if count == 0 {
                Err(ApplicationError::NotFound(format!(
                    "Portfolio project with id {} not found",
                    project_id_value
                )))
            } else {
                Ok(existing.resume_id)
            }
        }
        Err(err) => Err(app_err_from_diesel_err(err)),
    }
}

pub fn delete_portfolio_key_point(
    user_id_value: i32,
    key_point_id_value: i32,
) -> Result<i32, ApplicationError> {
    use domain::schema::portfolio_key_points;
    use domain::schema::portfolio_projects;

    let existing: PortfolioKeyPoint = match portfolio_key_points::table
        .find(key_point_id_value)
        .first(&mut establish_connection())
    {
        Ok(v) => v,
        Err(diesel::result::Error::NotFound) => {
            return Err(ApplicationError::NotFound(format!(
                "Portfolio key point with id {} not found",
                key_point_id_value
            )));
        }
        Err(err) => return Err(app_err_from_diesel_err(err)),
    };

    let project: PortfolioProject = match portfolio_projects::table
        .find(existing.portfolio_project_id)
        .first(&mut establish_connection())
    {
        Ok(p) => p,
        Err(diesel::result::Error::NotFound) => {
            return Err(ApplicationError::NotFound(
                "Portfolio project not found".to_string(),
            ));
        }
        Err(err) => return Err(app_err_from_diesel_err(err)),
    };

    let resume: Resume = match find_resume(project.resume_id) {
        Ok(r) => r,
        Err(err) => return Err(err),
    };

    match resume.created_by {
        Some(owner) if owner == user_id_value => {}
        Some(_) | None => {
            return Err(ApplicationError::Forbidden);
        }
    }

    match diesel::delete(portfolio_key_points::table.find(key_point_id_value))
        .execute(&mut establish_connection())
    {
        Ok(count) => {
            if count == 0 {
                Err(ApplicationError::NotFound(format!(
                    "Portfolio key point with id {} not found",
                    key_point_id_value
                )))
            } else {
                Ok(project.resume_id)
            }
        }
        Err(err) => Err(app_err_from_diesel_err(err)),
    }
}

pub fn delete_portfolio_technology(
    user_id_value: i32,
    tech_id_value: i32,
) -> Result<i32, ApplicationError> {
    use domain::schema::portfolio_projects;
    use domain::schema::portfolio_technologies;

    let existing: PortfolioTechnology = match portfolio_technologies::table
        .find(tech_id_value)
        .first(&mut establish_connection())
    {
        Ok(v) => v,
        Err(diesel::result::Error::NotFound) => {
            return Err(ApplicationError::NotFound(format!(
                "Portfolio technology with id {} not found",
                tech_id_value
            )));
        }
        Err(err) => return Err(app_err_from_diesel_err(err)),
    };

    let project: PortfolioProject = match portfolio_projects::table
        .find(existing.portfolio_project_id)
        .first(&mut establish_connection())
    {
        Ok(p) => p,
        Err(diesel::result::Error::NotFound) => {
            return Err(ApplicationError::NotFound(
                "Portfolio project not found".to_string(),
            ));
        }
        Err(err) => return Err(app_err_from_diesel_err(err)),
    };

    let resume: Resume = match find_resume(project.resume_id) {
        Ok(r) => r,
        Err(err) => return Err(err),
    };

    match resume.created_by {
        Some(owner) if owner == user_id_value => {}
        Some(_) | None => {
            return Err(ApplicationError::Forbidden);
        }
    }

    match diesel::delete(portfolio_technologies::table.find(tech_id_value))
        .execute(&mut establish_connection())
    {
        Ok(count) => {
            if count == 0 {
                Err(ApplicationError::NotFound(format!(
                    "Portfolio technology with id {} not found",
                    tech_id_value
                )))
            } else {
                Ok(project.resume_id)
            }
        }
        Err(err) => Err(app_err_from_diesel_err(err)),
    }
}
