use diesel::prelude::*;
use domain::models::{PortfolioKeyPoint, PortfolioProject, PortfolioTechnology};
use infrastructure::establish_connection;

use crate::{
    error::ApplicationError,
    resume::common::{app_err_from_diesel_err, find_accessible_resume},
};

pub fn list_portfolio_projects(
    resume_id_value: i32,
    user_id_value: Option<i32>,
) -> Result<Vec<PortfolioProject>, ApplicationError> {
    use domain::schema::portfolio_projects::dsl as projects_dsl;

    if let Err(err) = find_accessible_resume(resume_id_value, user_id_value) {
        return Err(err);
    }

    let mut items: Vec<PortfolioProject> = match projects_dsl::portfolio_projects
        .filter(projects_dsl::resume_id.eq(resume_id_value))
        .load::<PortfolioProject>(&mut establish_connection())
    {
        Ok(v) => v,
        Err(err) => return Err(app_err_from_diesel_err(err)),
    };

    items.sort_by_key(|p| (p.display_order.unwrap_or(0), p.id));

    Ok(items)
}

pub fn list_portfolio_key_points(
    resume_id_value: i32,
    project_id_value: i32,
    user_id_value: Option<i32>,
) -> Result<Vec<PortfolioKeyPoint>, ApplicationError> {
    use domain::schema::portfolio_key_points::dsl as kps_dsl;
    use domain::schema::portfolio_projects::dsl as projects_dsl;

    if let Err(err) = find_accessible_resume(resume_id_value, user_id_value) {
        return Err(err);
    }

    let _project: PortfolioProject = match projects_dsl::portfolio_projects
        .filter(projects_dsl::id.eq(project_id_value))
        .filter(projects_dsl::resume_id.eq(resume_id_value))
        .first(&mut establish_connection())
    {
        Ok(p) => p,
        Err(err) => return Err(app_err_from_diesel_err(err)),
    };

    let mut items: Vec<PortfolioKeyPoint> = match kps_dsl::portfolio_key_points
        .filter(kps_dsl::portfolio_project_id.eq(project_id_value))
        .load::<PortfolioKeyPoint>(&mut establish_connection())
    {
        Ok(v) => v,
        Err(err) => return Err(app_err_from_diesel_err(err)),
    };

    items.sort_by_key(|kp| (kp.display_order.unwrap_or(0), kp.id));

    Ok(items)
}

pub fn list_portfolio_technologies(
    resume_id_value: i32,
    project_id_value: i32,
    user_id_value: Option<i32>,
) -> Result<Vec<PortfolioTechnology>, ApplicationError> {
    use domain::schema::portfolio_projects::dsl as projects_dsl;
    use domain::schema::portfolio_technologies::dsl as tech_dsl;

    if let Err(err) = find_accessible_resume(resume_id_value, user_id_value) {
        return Err(err);
    }

    let _project: PortfolioProject = match projects_dsl::portfolio_projects
        .filter(projects_dsl::id.eq(project_id_value))
        .filter(projects_dsl::resume_id.eq(resume_id_value))
        .first(&mut establish_connection())
    {
        Ok(p) => p,
        Err(err) => return Err(app_err_from_diesel_err(err)),
    };

    let mut items: Vec<PortfolioTechnology> = match tech_dsl::portfolio_technologies
        .filter(tech_dsl::portfolio_project_id.eq(project_id_value))
        .load::<PortfolioTechnology>(&mut establish_connection())
    {
        Ok(v) => v,
        Err(err) => return Err(app_err_from_diesel_err(err)),
    };

    items.sort_by_key(|t| (t.display_order.unwrap_or(0), t.id));

    Ok(items)
}
