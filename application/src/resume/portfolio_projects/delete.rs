use diesel::prelude::*;
use domain::models::{PortfolioKeyPoint, PortfolioProject, PortfolioTechnology};

use crate::{
    error::ApplicationError,
    realtime::{ResumeChangePublisher, ResumeChangedAction, SectionType},
    resume::common::{first_or_404, require_owned_resume, run_published_delete},
};

pub fn delete_portfolio_project(
    user_id_value: i32,
    project_id_value: i32,
    publisher: &dyn ResumeChangePublisher,
) -> Result<(), ApplicationError> {
    use domain::schema::portfolio_projects;

    let existing: PortfolioProject = first_or_404(
        "Portfolio project",
        project_id_value,
        portfolio_projects::table.find(project_id_value),
    )?;

    require_owned_resume(user_id_value, existing.resume_id)?;

    run_published_delete(
        "Portfolio project",
        project_id_value,
        existing.resume_id,
        ResumeChangedAction::Updated(SectionType::Projects),
        publisher,
        |conn| diesel::delete(portfolio_projects::table.find(project_id_value)).execute(conn),
    )
}

pub fn delete_portfolio_key_point(
    user_id_value: i32,
    key_point_id_value: i32,
    publisher: &dyn ResumeChangePublisher,
) -> Result<(), ApplicationError> {
    use domain::schema::portfolio_key_points;
    use domain::schema::portfolio_projects;

    let existing: PortfolioKeyPoint = first_or_404(
        "Portfolio key point",
        key_point_id_value,
        portfolio_key_points::table.find(key_point_id_value),
    )?;

    let project: PortfolioProject = first_or_404(
        "Portfolio project",
        existing.portfolio_project_id,
        portfolio_projects::table.find(existing.portfolio_project_id),
    )?;

    require_owned_resume(user_id_value, project.resume_id)?;

    run_published_delete(
        "Portfolio key point",
        key_point_id_value,
        project.resume_id,
        ResumeChangedAction::Updated(SectionType::Projects),
        publisher,
        |conn| diesel::delete(portfolio_key_points::table.find(key_point_id_value)).execute(conn),
    )
}

pub fn delete_portfolio_technology(
    user_id_value: i32,
    tech_id_value: i32,
    publisher: &dyn ResumeChangePublisher,
) -> Result<(), ApplicationError> {
    use domain::schema::portfolio_projects;
    use domain::schema::portfolio_technologies;

    let existing: PortfolioTechnology = first_or_404(
        "Portfolio technology",
        tech_id_value,
        portfolio_technologies::table.find(tech_id_value),
    )?;

    let project: PortfolioProject = first_or_404(
        "Portfolio project",
        existing.portfolio_project_id,
        portfolio_projects::table.find(existing.portfolio_project_id),
    )?;

    require_owned_resume(user_id_value, project.resume_id)?;

    run_published_delete(
        "Portfolio technology",
        tech_id_value,
        project.resume_id,
        ResumeChangedAction::Updated(SectionType::Projects),
        publisher,
        |conn| diesel::delete(portfolio_technologies::table.find(tech_id_value)).execute(conn),
    )
}
