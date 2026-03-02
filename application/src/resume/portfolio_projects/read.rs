use diesel::prelude::*;
use domain::models::{PortfolioKeyPoint, PortfolioProject, PortfolioTechnology, Resume};
use infrastructure::establish_connection;

use crate::error::ApplicationError;

pub fn list_portfolio_projects(
    resume_id_value: i32,
    user_id_value: Option<i32>,
) -> Result<Vec<PortfolioProject>, ApplicationError> {
    use domain::schema::portfolio_projects::dsl as projects_dsl;
    use domain::schema::resumes::dsl as resumes_dsl;

    let mut resume_query = resumes_dsl::resumes.into_boxed();
    resume_query = resume_query.filter(resumes_dsl::id.eq(resume_id_value));
    resume_query = match user_id_value {
        Some(uid) => resume_query.filter(
            resumes_dsl::is_public
                .eq(true)
                .or(resumes_dsl::created_by.eq(uid)),
        ),
        None => resume_query.filter(resumes_dsl::is_public.eq(true)),
    };

    let _resume: Resume = match resume_query.first(&mut establish_connection()) {
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

    let mut items: Vec<PortfolioProject> = match projects_dsl::portfolio_projects
        .filter(projects_dsl::resume_id.eq(resume_id_value))
        .load::<PortfolioProject>(&mut establish_connection())
    {
        Ok(v) => v,
        Err(err) => {
            return Err(ApplicationError::Internal(format!(
                "Database error - {}",
                err
            )));
        }
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
    use domain::schema::resumes::dsl as resumes_dsl;

    let mut resume_query = resumes_dsl::resumes.into_boxed();
    resume_query = resume_query.filter(resumes_dsl::id.eq(resume_id_value));
    resume_query = match user_id_value {
        Some(uid) => resume_query.filter(
            resumes_dsl::is_public
                .eq(true)
                .or(resumes_dsl::created_by.eq(uid)),
        ),
        None => resume_query.filter(resumes_dsl::is_public.eq(true)),
    };

    let _resume: Resume = match resume_query.first(&mut establish_connection()) {
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

    let mut items: Vec<PortfolioKeyPoint> = match kps_dsl::portfolio_key_points
        .filter(kps_dsl::portfolio_project_id.eq(project_id_value))
        .load::<PortfolioKeyPoint>(&mut establish_connection())
    {
        Ok(v) => v,
        Err(err) => {
            return Err(ApplicationError::Internal(format!(
                "Database error - {}",
                err
            )));
        }
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
    use domain::schema::resumes::dsl as resumes_dsl;

    let mut resume_query = resumes_dsl::resumes.into_boxed();
    resume_query = resume_query.filter(resumes_dsl::id.eq(resume_id_value));
    resume_query = match user_id_value {
        Some(uid) => resume_query.filter(
            resumes_dsl::is_public
                .eq(true)
                .or(resumes_dsl::created_by.eq(uid)),
        ),
        None => resume_query.filter(resumes_dsl::is_public.eq(true)),
    };

    let _resume: Resume = match resume_query.first(&mut establish_connection()) {
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

    let mut items: Vec<PortfolioTechnology> = match tech_dsl::portfolio_technologies
        .filter(tech_dsl::portfolio_project_id.eq(project_id_value))
        .load::<PortfolioTechnology>(&mut establish_connection())
    {
        Ok(v) => v,
        Err(err) => {
            return Err(ApplicationError::Internal(format!(
                "Database error - {}",
                err
            )));
        }
    };

    items.sort_by_key(|t| (t.display_order.unwrap_or(0), t.id));

    Ok(items)
}
