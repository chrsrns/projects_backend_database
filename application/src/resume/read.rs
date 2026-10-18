use domain::models::{ResumeListParams, ResumeListQuery, ResumePage, ResumeView};

use crate::error::ApplicationError;
use crate::resume::view;
use uuid::Uuid;

pub fn list_resume(
    resume_id: Uuid,
    user_id_value: Option<i32>,
) -> Result<ResumeView, ApplicationError> {
    view::load_resume_view(resume_id, user_id_value)
}

pub fn list_resumes(
    user_id_value: Option<i32>,
    params: ResumeListParams,
) -> Result<ResumePage, ApplicationError> {
    let query = ResumeListQuery::try_from(params).map_err(ApplicationError::BadRequest)?;
    view::load_resume_views(user_id_value, &query)
}
