use diesel::prelude::*;
use domain::models::{
    PortfolioKeyPoint, PortfolioProject, PortfolioTechnology, Resume, UpdatePortfolioKeyPoint,
    UpdatePortfolioProject, UpdatePortfolioTechnology,
};
use infrastructure::establish_connection;

use crate::{
    error::ApplicationError,
    resume::common::{app_err_from_diesel_err, find_resume, validate_optional_url},
};

pub fn update_portfolio_project(
    user_id_value: i32,
    project_id_value: i32,
    mut payload: UpdatePortfolioProject,
) -> Result<PortfolioProject, ApplicationError> {
    use domain::schema::portfolio_projects;

    let existing: PortfolioProject = match portfolio_projects::table
        .find(project_id_value)
        .first(&mut establish_connection())
    {
        Ok(v) => v,
        Err(err) => return Err(app_err_from_diesel_err(err)),
    };

    let resume: Resume = find_resume(existing.resume_id)?;

    match resume.created_by {
        Some(owner) if owner == user_id_value => {}
        Some(_) | None => {
            return Err(ApplicationError::Forbidden);
        }
    }

    payload.video_url = match payload.video_url {
        None => None,
        Some(None) => Some(None),
        Some(Some(ref v)) if v.is_empty() => None,
        Some(Some(v)) => Some(validate_optional_url(Some(v), "Video URL")?),
    };

    payload.image_url = match payload.image_url {
        None => None,
        Some(None) => Some(None),
        Some(Some(v)) => Some(validate_optional_url(Some(v), "Image URL")?),
    };

    payload.project_link = match payload.project_link {
        None => None,
        Some(None) => Some(None),
        Some(Some(v)) => Some(validate_optional_url(Some(v), "Project Link")?),
    };

    payload.source_code_link = match payload.source_code_link {
        None => None,
        Some(None) => Some(None),
        Some(Some(v)) => Some(validate_optional_url(Some(v), "Source Code Link")?),
    };

    if payload.project_name.is_none()
        && payload.image_url.is_none()
        && payload.project_link.is_none()
        && payload.source_code_link.is_none()
        && payload.video_url.is_none()
        && payload.description.is_none()
        && payload.display_order.is_none()
        && payload.active.is_none()
    {
        return Ok(existing);
    }

    match diesel::update(portfolio_projects::table.find(project_id_value))
        .set(&payload)
        .get_result::<PortfolioProject>(&mut establish_connection())
    {
        Ok(updated) => Ok(updated),
        Err(err) => Err(app_err_from_diesel_err(err)),
    }
}

pub fn update_portfolio_key_point(
    user_id_value: i32,
    key_point_id_value: i32,
    payload: UpdatePortfolioKeyPoint,
) -> Result<(PortfolioKeyPoint, i32), ApplicationError> {
    use domain::schema::portfolio_key_points;
    use domain::schema::portfolio_projects;

    let existing: PortfolioKeyPoint = match portfolio_key_points::table
        .find(key_point_id_value)
        .first(&mut establish_connection())
    {
        Ok(v) => v,
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

    let resume: Resume = find_resume(project.resume_id)?;

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
        Err(err) => Err(app_err_from_diesel_err(err)),
    }
}

pub fn update_portfolio_technology(
    user_id_value: i32,
    tech_id_value: i32,
    payload: UpdatePortfolioTechnology,
) -> Result<(PortfolioTechnology, i32), ApplicationError> {
    use domain::schema::portfolio_projects;
    use domain::schema::portfolio_technologies;

    let existing: PortfolioTechnology = match portfolio_technologies::table
        .find(tech_id_value)
        .first(&mut establish_connection())
    {
        Ok(v) => v,
        Err(err) => return Err(app_err_from_diesel_err(err)),
    };

    let project: PortfolioProject = match portfolio_projects::table
        .find(existing.portfolio_project_id)
        .first(&mut establish_connection())
    {
        Ok(p) => p,
        Err(err) => return Err(app_err_from_diesel_err(err)),
    };

    let resume: Resume = find_resume(project.resume_id)?;

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
        Err(err) => Err(app_err_from_diesel_err(err)),
    }
}
