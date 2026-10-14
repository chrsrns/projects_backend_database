use diesel::prelude::*;
use domain::models::{
    PortfolioKeyPoint, PortfolioProject, PortfolioTechnology, UpdatePortfolioKeyPoint,
    UpdatePortfolioProject, UpdatePortfolioTechnology,
};
use infrastructure::establish_connection;

use crate::{
    error::ApplicationError,
    realtime::{ResumeChangePublisher, ResumeChangedAction, SectionType},
    resume::common::{
        app_err_from_diesel_err, first_or_404, require_owned_resume, validate_optional_url_update,
    },
};

pub fn update_portfolio_project(
    user_id_value: i32,
    project_id_value: i32,
    mut payload: UpdatePortfolioProject,
    publisher: &dyn ResumeChangePublisher,
) -> Result<PortfolioProject, ApplicationError> {
    use domain::schema::portfolio_projects;

    let existing: PortfolioProject = first_or_404(
        "Portfolio project",
        project_id_value,
        portfolio_projects::table.find(project_id_value),
    )?;

    require_owned_resume(user_id_value, existing.resume_id)?;

    payload.video_url = validate_optional_url_update(payload.video_url, "Video URL")?;
    payload.image_url = validate_optional_url_update(payload.image_url, "Image URL")?;
    payload.project_link = validate_optional_url_update(payload.project_link, "Project Link")?;
    payload.source_code_link =
        validate_optional_url_update(payload.source_code_link, "Source Code Link")?;

    if payload.is_noop() {
        publisher.resume_changed(
            existing.resume_id,
            ResumeChangedAction::Updated(SectionType::Projects),
        );
        return Ok(existing);
    }

    match diesel::update(portfolio_projects::table.find(project_id_value))
        .set(&payload)
        .get_result::<PortfolioProject>(&mut establish_connection())
    {
        Ok(updated) => {
            publisher.resume_changed(
                updated.resume_id,
                ResumeChangedAction::Updated(SectionType::Projects),
            );
            Ok(updated)
        }
        Err(err) => Err(app_err_from_diesel_err(err)),
    }
}

pub fn update_portfolio_key_point(
    user_id_value: i32,
    key_point_id_value: i32,
    payload: UpdatePortfolioKeyPoint,
    publisher: &dyn ResumeChangePublisher,
) -> Result<(PortfolioKeyPoint, i32), ApplicationError> {
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

    match diesel::update(portfolio_key_points::table.find(key_point_id_value))
        .set(&payload)
        .get_result::<PortfolioKeyPoint>(&mut establish_connection())
    {
        Ok(updated) => {
            publisher.resume_changed(
                project.resume_id,
                ResumeChangedAction::Updated(SectionType::Projects),
            );
            Ok((updated, project.resume_id))
        }
        Err(err) => Err(app_err_from_diesel_err(err)),
    }
}

pub fn update_portfolio_technology(
    user_id_value: i32,
    tech_id_value: i32,
    payload: UpdatePortfolioTechnology,
    publisher: &dyn ResumeChangePublisher,
) -> Result<(PortfolioTechnology, i32), ApplicationError> {
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

    match diesel::update(portfolio_technologies::table.find(tech_id_value))
        .set(&payload)
        .get_result::<PortfolioTechnology>(&mut establish_connection())
    {
        Ok(updated) => {
            publisher.resume_changed(
                project.resume_id,
                ResumeChangedAction::Updated(SectionType::Projects),
            );
            Ok((updated, project.resume_id))
        }
        Err(diesel::result::Error::NotFound) => Err(ApplicationError::NotFound(format!(
            "Portfolio technology with id {} not found",
            tech_id_value
        ))),
        Err(err) => Err(app_err_from_diesel_err(err)),
    }
}
