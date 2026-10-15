use domain::models::ResumeView;

use crate::error::ApplicationError;
use crate::resume::view;
use uuid::Uuid;

pub fn list_resume(
    resume_id: Uuid,
    user_id_value: Option<i32>,
) -> Result<ResumeView, ApplicationError> {
    view::load_resume_view(resume_id, user_id_value)
}

pub fn list_resumes(user_id_value: Option<i32>) -> Result<Vec<ResumeView>, ApplicationError> {
    view::load_resume_views(user_id_value)
}
