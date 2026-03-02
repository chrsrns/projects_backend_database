use diesel::prelude::*;
use domain::models::{
    PortfolioKeyPoint, PortfolioProject, PortfolioTechnology, Resume, UpdatePortfolioKeyPoint,
    UpdatePortfolioProject, UpdatePortfolioTechnology,
};
use infrastructure::establish_connection;

use crate::error::ApplicationError;

pub fn update_portfolio_project(
    user_id_value: i32,
    project_id_value: i32,
    payload: UpdatePortfolioProject,
) -> Result<PortfolioProject, ApplicationError> {
    use domain::schema::portfolio_projects;
    use domain::schema::resumes;

    let existing: PortfolioProject = match portfolio_projects::table
        .find(project_id_value)
        .first(&mut establish_connection())
    {
        Ok(v) => v,
        Err(diesel::result::Error::NotFound) => {
            return Err(ApplicationError::NotFound(format!(
                "Portfolio project with id {} not found",
                project_id_value
            )));
        }
        Err(err) => {
            return Err(ApplicationError::Internal(format!(
                "Database error - {}",
                err
            )));
        }
    };

    let resume: Resume = match resumes::table
        .find(existing.resume_id)
        .first(&mut establish_connection())
    {
        Ok(r) => r,
        Err(diesel::result::Error::NotFound) => {
            return Err(ApplicationError::NotFound("Resume not found".to_string()));
        }
        Err(err) => {
            return Err(ApplicationError::Internal(format!(
                "Database error - {}",
                err
            )));
        }
    };

    match resume.created_by {
        Some(owner) if owner == user_id_value => {}
        Some(_) | None => {
            return Err(ApplicationError::Forbidden);
        }
    }

    match diesel::update(portfolio_projects::table.find(project_id_value))
        .set(&payload)
        .get_result::<PortfolioProject>(&mut establish_connection())
    {
        Ok(updated) => Ok(updated),
        Err(diesel::result::Error::NotFound) => Err(ApplicationError::NotFound(format!(
            "Portfolio project with id {} not found",
            project_id_value
        ))),
        Err(err) => Err(ApplicationError::Internal(format!(
            "Database error - {}",
            err
        ))),
    }
}

pub fn update_portfolio_key_point(
    user_id_value: i32,
    key_point_id_value: i32,
    payload: UpdatePortfolioKeyPoint,
) -> Result<(PortfolioKeyPoint, i32), ApplicationError> {
    use domain::schema::portfolio_key_points;
    use domain::schema::portfolio_projects;
    use domain::schema::resumes;

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
        Err(err) => {
            return Err(ApplicationError::Internal(format!(
                "Database error - {}",
                err
            )));
        }
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
        Err(err) => {
            return Err(ApplicationError::Internal(format!(
                "Database error - {}",
                err
            )));
        }
    };

    let resume: Resume = match resumes::table
        .find(project.resume_id)
        .first(&mut establish_connection())
    {
        Ok(r) => r,
        Err(diesel::result::Error::NotFound) => {
            return Err(ApplicationError::NotFound("Resume not found".to_string()));
        }
        Err(err) => {
            return Err(ApplicationError::Internal(format!(
                "Database error - {}",
                err
            )));
        }
    };

    match resume.created_by {
        Some(owner) if owner == user_id_value => {}
        Some(_) | None => {
            return Err(ApplicationError::Forbidden);
        }
    }

    match diesel::update(portfolio_key_points::table.find(key_point_id_value))
        .set(&payload)
        .get_result::<PortfolioKeyPoint>(&mut establish_connection())
    {
        Ok(updated) => Ok((updated, project.resume_id)),
        Err(diesel::result::Error::NotFound) => Err(ApplicationError::NotFound(format!(
            "Portfolio key point with id {} not found",
            key_point_id_value
        ))),
        Err(err) => Err(ApplicationError::Internal(format!(
            "Database error - {}",
            err
        ))),
    }
}

pub fn update_portfolio_technology(
    user_id_value: i32,
    tech_id_value: i32,
    payload: UpdatePortfolioTechnology,
) -> Result<(PortfolioTechnology, i32), ApplicationError> {
    use domain::schema::portfolio_projects;
    use domain::schema::portfolio_technologies;
    use domain::schema::resumes;

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
        Err(err) => {
            return Err(ApplicationError::Internal(format!(
                "Database error - {}",
                err
            )));
        }
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
        Err(err) => {
            return Err(ApplicationError::Internal(format!(
                "Database error - {}",
                err
            )));
        }
    };

    let resume: Resume = match resumes::table
        .find(project.resume_id)
        .first(&mut establish_connection())
    {
        Ok(r) => r,
        Err(diesel::result::Error::NotFound) => {
            return Err(ApplicationError::NotFound("Resume not found".to_string()));
        }
        Err(err) => {
            return Err(ApplicationError::Internal(format!(
                "Database error - {}",
                err
            )));
        }
    };

    match resume.created_by {
        Some(owner) if owner == user_id_value => {}
        Some(_) | None => {
            return Err(ApplicationError::Forbidden);
        }
    }

    match diesel::update(portfolio_technologies::table.find(tech_id_value))
        .set(&payload)
        .get_result::<PortfolioTechnology>(&mut establish_connection())
    {
        Ok(updated) => Ok((updated, project.resume_id)),
        Err(diesel::result::Error::NotFound) => Err(ApplicationError::NotFound(format!(
            "Portfolio technology with id {} not found",
            tech_id_value
        ))),
        Err(err) => Err(ApplicationError::Internal(format!(
            "Database error - {}",
            err
        ))),
    }
}
