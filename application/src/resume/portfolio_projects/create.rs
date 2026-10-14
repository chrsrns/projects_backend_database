use diesel::prelude::*;
use domain::models::{
    NewPortfolioKeyPoint, NewPortfolioKeyPointRequest, NewPortfolioProject,
    NewPortfolioProjectRequest, NewPortfolioTechnology, NewPortfolioTechnologyRequest,
    PortfolioKeyPoint, PortfolioProject, PortfolioTechnology,
};

use crate::{
    error::ApplicationError,
    realtime::{ResumeChangePublisher, ResumeChangedAction, SectionType},
    resume::common::{first_or_404, require_owned_resume, run_published, validate_optional_url},
};

pub fn create_portfolio_project(
    user_id_value: i32,
    resume_id_value: i32,
    payload: NewPortfolioProjectRequest,
    publisher: &dyn ResumeChangePublisher,
) -> Result<PortfolioProject, ApplicationError> {
    use domain::schema::portfolio_projects;

    require_owned_resume(user_id_value, resume_id_value)?;

    let new_item = NewPortfolioProject {
        resume_id: resume_id_value,
        project_name: payload.project_name,
        image_url: validate_optional_url(payload.image_url, "Image URL")?,
        project_link: validate_optional_url(payload.project_link, "Project Link")?,
        source_code_link: validate_optional_url(payload.source_code_link, "Source Code Link")?,
        video_url: validate_optional_url(payload.video_url, "Video URL")?,
        description: payload.description,
        display_order: payload.display_order,
        active: true,
    };

    run_published(
        "Portfolio project",
        0,
        resume_id_value,
        ResumeChangedAction::Updated(SectionType::Projects),
        publisher,
        |conn| {
            diesel::insert_into(portfolio_projects::table)
                .values(&new_item)
                .get_result::<PortfolioProject>(conn)
        },
    )
}

pub fn create_portfolio_key_point(
    user_id_value: i32,
    resume_id_value: i32,
    project_id_value: i32,
    payload: NewPortfolioKeyPointRequest,
    publisher: &dyn ResumeChangePublisher,
) -> Result<PortfolioKeyPoint, ApplicationError> {
    use domain::schema::portfolio_key_points;
    use domain::schema::portfolio_projects::dsl as projects_dsl;

    require_owned_resume(user_id_value, resume_id_value)?;

    let _project: PortfolioProject = first_or_404(
        "Portfolio project",
        project_id_value,
        projects_dsl::portfolio_projects
            .filter(projects_dsl::id.eq(project_id_value))
            .filter(projects_dsl::resume_id.eq(resume_id_value)),
    )?;

    let new_item = NewPortfolioKeyPoint {
        portfolio_project_id: project_id_value,
        key_point: payload.key_point,
        display_order: payload.display_order,
        active: true,
    };

    run_published(
        "Portfolio key point",
        0,
        resume_id_value,
        ResumeChangedAction::Updated(SectionType::Projects),
        publisher,
        |conn| {
            diesel::insert_into(portfolio_key_points::table)
                .values(&new_item)
                .get_result::<PortfolioKeyPoint>(conn)
        },
    )
}

pub fn create_portfolio_technology(
    user_id_value: i32,
    resume_id_value: i32,
    project_id_value: i32,
    payload: NewPortfolioTechnologyRequest,
    publisher: &dyn ResumeChangePublisher,
) -> Result<PortfolioTechnology, ApplicationError> {
    use domain::schema::portfolio_projects::dsl as projects_dsl;
    use domain::schema::portfolio_technologies;

    require_owned_resume(user_id_value, resume_id_value)?;

    let _project: PortfolioProject = first_or_404(
        "Portfolio project",
        project_id_value,
        projects_dsl::portfolio_projects
            .filter(projects_dsl::id.eq(project_id_value))
            .filter(projects_dsl::resume_id.eq(resume_id_value)),
    )?;

    let new_item = NewPortfolioTechnology {
        portfolio_project_id: project_id_value,
        technology_name: payload.technology_name,
        display_order: payload.display_order,
        active: true,
    };

    run_published(
        "Portfolio technology",
        0,
        resume_id_value,
        ResumeChangedAction::Updated(SectionType::Projects),
        publisher,
        |conn| {
            diesel::insert_into(portfolio_technologies::table)
                .values(&new_item)
                .get_result::<PortfolioTechnology>(conn)
        },
    )
}
