use diesel::prelude::*;
use domain::models::{Resume, Skill, UpdateSkill};
use infrastructure::establish_connection;

use crate::{
    error::ApplicationError,
    realtime::{ResumeChangePublisher, ResumeChangedAction, SectionType},
    resume::common::{app_err_from_diesel_err, find_resume, require_owner},
};

pub fn update_skill(
    user_id_value: i32,
    skill_id_value: i32,
    payload: UpdateSkill,
    publisher: &dyn ResumeChangePublisher,
) -> Result<Skill, ApplicationError> {
    use domain::schema::skills;

    let existing: Skill = match skills::table
        .find(skill_id_value)
        .first(&mut establish_connection())
    {
        Ok(r) => r,
        Err(diesel::result::Error::NotFound) => {
            return Err(ApplicationError::NotFound(format!(
                "Skill with id {} not found",
                skill_id_value
            )));
        }
        Err(err) => return Err(app_err_from_diesel_err(err)),
    };

    let resume: Resume = find_resume(existing.resume_id)?;

    require_owner(&resume, user_id_value)?;

    match diesel::update(skills::table.find(skill_id_value))
        .set(&payload)
        .get_result::<Skill>(&mut establish_connection())
    {
        Ok(updated) => {
            publisher.resume_changed(
                updated.resume_id,
                ResumeChangedAction::Updated(SectionType::Skills),
            );
            Ok(updated)
        }
        Err(err) => Err(app_err_from_diesel_err(err)),
    }
}
