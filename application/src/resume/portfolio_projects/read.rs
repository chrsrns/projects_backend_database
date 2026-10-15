use diesel::prelude::*;
use domain::models::{PortfolioKeyPoint, PortfolioProject, PortfolioTechnology};
use infrastructure::establish_connection;

use uuid::Uuid;

use crate::{
    error::ApplicationError,
    resume::common::{
        app_err_from_diesel_err, find_accessible_resume, first_or_404, viewer_is_owner,
    },
};

pub fn list_portfolio_projects(
    resume_id_value: Uuid,
    user_id_value: Option<i32>,
) -> Result<Vec<PortfolioProject>, ApplicationError> {
    use domain::schema::portfolio_projects::dsl as projects_dsl;

    let resume = find_accessible_resume(resume_id_value, user_id_value)?;

    let is_owner = viewer_is_owner(&resume, user_id_value);

    let mut query = projects_dsl::portfolio_projects.into_boxed();
    query = query.filter(projects_dsl::resume_id.eq(resume_id_value));
    if !is_owner {
        query = query.filter(projects_dsl::active.eq(true));
    }

    let mut items: Vec<PortfolioProject> =
        match query.load::<PortfolioProject>(&mut establish_connection()) {
            Ok(v) => v,
            Err(err) => return Err(app_err_from_diesel_err(err)),
        };

    items.sort_by_key(|p| (p.display_order.unwrap_or(0), p.id));

    Ok(items)
}

pub fn list_portfolio_key_points(
    resume_id_value: Uuid,
    project_id_value: i32,
    user_id_value: Option<i32>,
) -> Result<Vec<PortfolioKeyPoint>, ApplicationError> {
    use domain::schema::portfolio_key_points::dsl as kps_dsl;
    use domain::schema::portfolio_projects::dsl as projects_dsl;

    let resume = find_accessible_resume(resume_id_value, user_id_value)?;

    let is_owner = viewer_is_owner(&resume, user_id_value);

    let mut project_query = projects_dsl::portfolio_projects.into_boxed();
    project_query = project_query
        .filter(projects_dsl::id.eq(project_id_value))
        .filter(projects_dsl::resume_id.eq(resume_id_value));
    if !is_owner {
        project_query = project_query.filter(projects_dsl::active.eq(true));
    }

    let _project: PortfolioProject =
        first_or_404("Portfolio project", project_id_value, project_query)?;

    let mut kp_query = kps_dsl::portfolio_key_points.into_boxed();
    kp_query = kp_query.filter(kps_dsl::portfolio_project_id.eq(project_id_value));
    if !is_owner {
        kp_query = kp_query.filter(kps_dsl::active.eq(true));
    }

    let mut items: Vec<PortfolioKeyPoint> =
        match kp_query.load::<PortfolioKeyPoint>(&mut establish_connection()) {
            Ok(v) => v,
            Err(err) => return Err(app_err_from_diesel_err(err)),
        };

    items.sort_by_key(|kp| (kp.display_order.unwrap_or(0), kp.id));

    Ok(items)
}

pub fn list_portfolio_technologies(
    resume_id_value: Uuid,
    project_id_value: i32,
    user_id_value: Option<i32>,
) -> Result<Vec<PortfolioTechnology>, ApplicationError> {
    use domain::schema::portfolio_projects::dsl as projects_dsl;
    use domain::schema::portfolio_technologies::dsl as tech_dsl;

    let resume = find_accessible_resume(resume_id_value, user_id_value)?;

    let is_owner = viewer_is_owner(&resume, user_id_value);

    let mut project_query = projects_dsl::portfolio_projects.into_boxed();
    project_query = project_query
        .filter(projects_dsl::id.eq(project_id_value))
        .filter(projects_dsl::resume_id.eq(resume_id_value));
    if !is_owner {
        project_query = project_query.filter(projects_dsl::active.eq(true));
    }

    let _project: PortfolioProject =
        first_or_404("Portfolio project", project_id_value, project_query)?;

    let mut tech_query = tech_dsl::portfolio_technologies.into_boxed();
    tech_query = tech_query.filter(tech_dsl::portfolio_project_id.eq(project_id_value));
    if !is_owner {
        tech_query = tech_query.filter(tech_dsl::active.eq(true));
    }

    let mut items: Vec<PortfolioTechnology> =
        match tech_query.load::<PortfolioTechnology>(&mut establish_connection()) {
            Ok(v) => v,
            Err(err) => return Err(app_err_from_diesel_err(err)),
        };

    items.sort_by_key(|t| (t.display_order.unwrap_or(0), t.id));

    Ok(items)
}
