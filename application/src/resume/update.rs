use diesel::prelude::*;
use domain::models::{Resume, UpdateResume};
use domain::schema::resumes;
use infrastructure::establish_connection;

use crate::error::ApplicationError;
use crate::resume::common::{
    app_err_from_diesel_err, find_resume, validate_executive_summary, validate_video,
};

pub fn update_resume(
    user_id_value: i32,
    resume_id: i32,
    mut resume: UpdateResume,
) -> Result<Resume, ApplicationError> {
    let existing = find_resume(resume_id)?;

    match existing.created_by {
        Some(owner) if owner == user_id_value => {}
        Some(_) | None => {
            return Err(ApplicationError::Forbidden);
        }
    }

    resume.executive_summary = resume
        .executive_summary
        .map(validate_executive_summary)
        .transpose()?;

    resume.video = match resume.video {
        None => None,
        Some(None) => Some(None),
        Some(Some(ref v)) if v.is_empty() => None,
        Some(Some(v)) => Some(validate_video(Some(v))?),
    };

    match diesel::update(resumes::table.find(resume_id))
        .set(&resume)
        .get_result::<Resume>(&mut establish_connection())
    {
        Ok(updated_resume) => Ok(updated_resume),
        Err(err) => Err(app_err_from_diesel_err(err)),
    }
}
