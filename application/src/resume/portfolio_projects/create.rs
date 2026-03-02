use diesel::prelude::*;
use domain::models::{
    NewPortfolioKeyPoint, NewPortfolioKeyPointRequest, NewPortfolioProject,
    NewPortfolioProjectRequest, NewPortfolioTechnology, NewPortfolioTechnologyRequest,
    PortfolioKeyPoint, PortfolioProject, PortfolioTechnology, Resume,
};
use infrastructure::establish_connection;

use crate::error::ApplicationError;

pub fn create_portfolio_project(
    user_id_value: i32,
    resume_id_value: i32,
    payload: NewPortfolioProjectRequest,
) -> Result<PortfolioProject, ApplicationError> {
    use domain::schema::portfolio_projects;
    use domain::schema::resumes;

    let resume: Resume = match resumes::table
        .find(resume_id_value)
        .first(&mut establish_connection())
    {
        Ok(r) => r,
        Err(diesel::result::Error::NotFound) => {
            return Err(ApplicationError::NotFound(format!(
                "Resume with id {} not found",
                resume_id_value
            )));
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

    let new_item = NewPortfolioProject {
        resume_id: resume_id_value,
        project_name: payload.project_name,
        image_url: payload.image_url,
        project_link: payload.project_link,
        source_code_link: payload.source_code_link,
        description: payload.description,
        display_order: payload.display_order,
    };

    match diesel::insert_into(portfolio_projects::table)
        .values(&new_item)
        .get_result::<PortfolioProject>(&mut establish_connection())
    {
        Ok(item) => Ok(item),
        Err(err) => Err(ApplicationError::Internal(format!(
            "Database error - {}",
            err
        ))),
    }
}

pub fn create_portfolio_key_point(
    user_id_value: i32,
    resume_id_value: i32,
    project_id_value: i32,
    payload: NewPortfolioKeyPointRequest,
) -> Result<PortfolioKeyPoint, ApplicationError> {
    use domain::schema::portfolio_key_points;
    use domain::schema::portfolio_projects::dsl as projects_dsl;
    use domain::schema::resumes;

    let resume: Resume = match resumes::table
        .find(resume_id_value)
        .first(&mut establish_connection())
    {
        Ok(r) => r,
        Err(diesel::result::Error::NotFound) => {
            return Err(ApplicationError::NotFound(format!(
                "Resume with id {} not found",
                resume_id_value
            )));
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

    let _project: PortfolioProject = match projects_dsl::portfolio_projects
        .filter(projects_dsl::id.eq(project_id_value))
        .filter(projects_dsl::resume_id.eq(resume_id_value))
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

    let new_item = NewPortfolioKeyPoint {
        portfolio_project_id: project_id_value,
        key_point: payload.key_point,
        display_order: payload.display_order,
    };

    match diesel::insert_into(portfolio_key_points::table)
        .values(&new_item)
        .get_result::<PortfolioKeyPoint>(&mut establish_connection())
    {
        Ok(item) => Ok(item),
        Err(err) => Err(ApplicationError::Internal(format!(
            "Database error - {}",
            err
        ))),
    }
}

pub fn create_portfolio_technology(
    user_id_value: i32,
    resume_id_value: i32,
    project_id_value: i32,
    payload: NewPortfolioTechnologyRequest,
) -> Result<PortfolioTechnology, ApplicationError> {
    use domain::schema::portfolio_projects::dsl as projects_dsl;
    use domain::schema::portfolio_technologies;
    use domain::schema::resumes;

    let resume: Resume = match resumes::table
        .find(resume_id_value)
        .first(&mut establish_connection())
    {
        Ok(r) => r,
        Err(diesel::result::Error::NotFound) => {
            return Err(ApplicationError::NotFound(format!(
                "Resume with id {} not found",
                resume_id_value
            )));
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

    let _project: PortfolioProject = match projects_dsl::portfolio_projects
        .filter(projects_dsl::id.eq(project_id_value))
        .filter(projects_dsl::resume_id.eq(resume_id_value))
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

    let new_item = NewPortfolioTechnology {
        portfolio_project_id: project_id_value,
        technology_name: payload.technology_name,
        display_order: payload.display_order,
    };

    match diesel::insert_into(portfolio_technologies::table)
        .values(&new_item)
        .get_result::<PortfolioTechnology>(&mut establish_connection())
    {
        Ok(item) => Ok(item),
        Err(err) => Err(ApplicationError::Internal(format!(
            "Database error - {}",
            err
        ))),
    }
}
