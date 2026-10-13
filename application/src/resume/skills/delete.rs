use diesel::prelude::*;
use domain::models::{Resume, Skill};
use infrastructure::establish_connection;

use crate::{
    error::ApplicationError,
    realtime::{ResumeChangePublisher, ResumeChangedAction, SectionType},
    resume::common::{app_err_from_diesel_err, find_resume, require_owner},
};

pub fn delete_skill(
    user_id_value: i32,
    skill_id_value: i32,
    publisher: &dyn ResumeChangePublisher,
) -> Result<i32, ApplicationError> {
    use domain::schema::skills;

    let existing: Skill = match skills::table
        .find(skill_id_value)
        .first(&mut establish_connection())
    {
        Ok(r) => r,
        Err(err) => return Err(app_err_from_diesel_err(err)),
    };

    let resume: Resume = find_resume(existing.resume_id)?;

    require_owner(&resume, user_id_value)?;

    match diesel::delete(skills::table.find(skill_id_value)).execute(&mut establish_connection()) {
        Ok(count) => {
            if count == 0 {
                Err(ApplicationError::NotFound(format!(
                    "Skill with id {} not found",
                    skill_id_value
                )))
            } else {
                publisher.resume_changed(
                    existing.resume_id,
                    ResumeChangedAction::Updated(SectionType::Skills),
                );
                Ok(existing.resume_id)
            }
        }
        Err(err) => Err(app_err_from_diesel_err(err)),
    }
}
