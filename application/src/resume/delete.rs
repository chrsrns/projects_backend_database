use diesel::prelude::*;
use infrastructure::establish_connection;

use crate::{
    error::ApplicationError,
    realtime::{ResumeChangePublisher, ResumeChangedAction},
    resume::common::{app_err_from_diesel_err, require_owned_resume},
};

pub fn delete_resume(
    user_id_value: i32,
    resume_id: i32,
    publisher: &dyn ResumeChangePublisher,
) -> Result<(), ApplicationError> {
    use domain::schema::resumes;

    require_owned_resume(user_id_value, resume_id)?;

    let variant_count: i64 = match resumes::table
        .filter(resumes::base_resume_id.eq(resume_id))
        .count()
        .get_result(&mut establish_connection())
    {
        Ok(count) => count,
        Err(err) => return Err(app_err_from_diesel_err(err)),
    };

    if variant_count > 0 {
        return Err(ApplicationError::Conflict(format!(
            "Resume with id {} has {} variant(s) and cannot be deleted",
            resume_id, variant_count
        )));
    }

    match diesel::delete(resumes::table.find(resume_id)).execute(&mut establish_connection()) {
        Ok(count) => {
            if count == 0 {
                Err(ApplicationError::NotFound(format!(
                    "Resume with id {} not found",
                    resume_id
                )))
            } else {
                publisher.resume_changed(resume_id, ResumeChangedAction::Deleted);
                Ok(())
            }
        }
        Err(err) => Err(app_err_from_diesel_err(err)),
    }
}
