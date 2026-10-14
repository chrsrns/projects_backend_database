use diesel::prelude::*;
use domain::models::Skill;
use infrastructure::establish_connection;

use crate::{
    error::ApplicationError,
    realtime::{ResumeChangePublisher, ResumeChangedAction, SectionType},
    resume::common::{app_err_from_diesel_err, first_or_404, require_owned_resume},
};

pub fn delete_skill(
    user_id_value: i32,
    skill_id_value: i32,
    publisher: &dyn ResumeChangePublisher,
) -> Result<i32, ApplicationError> {
    use domain::schema::skills;

    let existing: Skill =
        first_or_404("Skill", skill_id_value, skills::table.find(skill_id_value))?;

    require_owned_resume(user_id_value, existing.resume_id)?;

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
