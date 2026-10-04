use std::collections::HashMap;

use diesel::alias;
use diesel::prelude::*;
use domain::models::{Resume, ResumeView};
use domain::schema::resumes;
use infrastructure::establish_connection;

use crate::error::ApplicationError;
use crate::resume::common::app_err_from_diesel_err;

alias!(resumes as base_resume: BaseResume, resumes as joined_resume: JoinedResume);

/// Maps each resume id to whether the viewer may read the base resume it
/// points at. Resolved with a single self-join for the whole page rather
/// than one lookup per row.
fn base_accessibility(
    viewer: Option<i32>,
    resume_ids: &[i32],
) -> Result<HashMap<i32, bool>, ApplicationError> {
    if resume_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let rows: Vec<(Option<i32>, bool, Option<i32>)> = joined_resume
        .inner_join(
            base_resume.on(base_resume
                .field(resumes::id)
                .nullable()
                .eq(joined_resume.field(resumes::base_resume_id))),
        )
        .filter(joined_resume.field(resumes::id).eq_any(resume_ids))
        .select((
            joined_resume.field(resumes::base_resume_id),
            base_resume.field(resumes::is_public),
            base_resume.field(resumes::created_by),
        ))
        .load(&mut establish_connection())
        .map_err(app_err_from_diesel_err)?;

    let mut accessibility = HashMap::new();
    for (base_id, base_is_public, base_owner) in rows {
        if let Some(base_id) = base_id {
            let reachable = base_is_public
                || (viewer.is_some() && base_owner.is_some() && base_owner == viewer);
            accessibility.insert(base_id, reachable);
        }
    }

    Ok(accessibility)
}

fn visible_resumes(
    viewer: Option<i32>,
    resume_id: Option<i32>,
) -> Result<Vec<Resume>, ApplicationError> {
    use domain::schema::resumes::dsl as resumes_dsl;

    let mut query = resumes_dsl::resumes.into_boxed();
    query = match viewer {
        Some(user_id) => query.filter(
            resumes_dsl::is_public
                .eq(true)
                .or(resumes_dsl::created_by.eq(user_id)),
        ),
        None => query.filter(resumes_dsl::is_public.eq(true)),
    };
    if let Some(resume_id) = resume_id {
        query = query.filter(resumes_dsl::id.eq(resume_id));
    }

    query
        .select(resumes::all_columns)
        .order(resumes_dsl::id.asc())
        .load::<Resume>(&mut establish_connection())
        .map_err(app_err_from_diesel_err)
}

fn build_views(
    viewer: Option<i32>,
    items: Vec<Resume>,
) -> Result<Vec<ResumeView>, ApplicationError> {
    let resume_ids: Vec<i32> = items.iter().map(|resume| resume.id).collect();
    let accessibility = base_accessibility(viewer, &resume_ids)?;

    Ok(items
        .into_iter()
        .map(|resume| {
            let is_owner = viewer.is_some() && resume.created_by == viewer;
            let base_reachable = resume
                .base_resume_id
                .and_then(|base_id| accessibility.get(&base_id).copied())
                .unwrap_or(false);
            ResumeView::from_resume(resume, base_reachable, is_owner)
        })
        .collect())
}

pub fn load_resume_view(
    resume_id: i32,
    viewer: Option<i32>,
) -> Result<ResumeView, ApplicationError> {
    let mut items = visible_resumes(viewer, Some(resume_id))?;

    match items.pop() {
        Some(resume) => Ok(build_views(viewer, vec![resume])?
            .pop()
            .expect("a single resume builds a single view")),
        None => Err(ApplicationError::NotFound(format!(
            "Resume with id {} not found",
            resume_id
        ))),
    }
}

pub fn load_resume_views(viewer: Option<i32>) -> Result<Vec<ResumeView>, ApplicationError> {
    build_views(viewer, visible_resumes(viewer, None)?)
}

/// Builds views for rows the caller has already proven they may read, such
/// as a freshly created resume or the variants of an accessible base.
pub fn views_for_known_accessible_base(
    items: Vec<Resume>,
    viewer: Option<i32>,
    base_reachable: bool,
) -> Vec<ResumeView> {
    items
        .into_iter()
        .map(|resume| {
            let is_owner = viewer.is_some() && resume.created_by == viewer;
            ResumeView::from_resume(resume, base_reachable, is_owner)
        })
        .collect()
}
