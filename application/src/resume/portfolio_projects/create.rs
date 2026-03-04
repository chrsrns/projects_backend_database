use diesel::prelude::*;
use domain::models::{
    NewPortfolioKeyPoint, NewPortfolioKeyPointRequest, NewPortfolioProject,
    NewPortfolioProjectRequest, NewPortfolioTechnology, NewPortfolioTechnologyRequest,
    PortfolioKeyPoint, PortfolioProject, PortfolioTechnology, Resume,
};
use infrastructure::establish_connection;

use crate::{
    error::ApplicationError,
    resume::common::{app_err_from_diesel_err, find_resume},
};

pub fn create_portfolio_project(
    user_id_value: i32,
    resume_id_value: i32,
    payload: NewPortfolioProjectRequest,
) -> Result<PortfolioProject, ApplicationError> {
    use domain::schema::portfolio_projects;

    let resume: Resume = match find_resume(resume_id_value) {
        Ok(r) => r,
        Err(err) => return Err(err),
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
        Err(err) => Err(app_err_from_diesel_err(err)),
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

    let resume: Resume = match find_resume(resume_id_value) {
        Ok(r) => r,
        Err(err) => return Err(err),
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
        Err(err) => {
            return Err(app_err_from_diesel_err(err));
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
        Err(err) => Err(app_err_from_diesel_err(err)),
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

    let resume: Resume = match find_resume(resume_id_value) {
        Ok(r) => r,
        Err(err) => return Err(err),
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
        Err(err) => return Err(app_err_from_diesel_err(err)),
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
        Err(err) => Err(app_err_from_diesel_err(err)),
    }
}
