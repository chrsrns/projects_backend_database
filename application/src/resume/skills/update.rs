use diesel::prelude::*;
use domain::models::{Skill, UpdateSkill};
use infrastructure::establish_connection;

use crate::{
    error::ApplicationError,
    realtime::{ResumeChangePublisher, ResumeChangedAction, SectionType},
    resume::common::{app_err_from_diesel_err, first_or_404, require_owned_resume},
};

pub fn update_skill(
    user_id_value: i32,
    skill_id_value: i32,
    payload: UpdateSkill,
    publisher: &dyn ResumeChangePublisher,
) -> Result<Skill, ApplicationError> {
    use domain::schema::skills;

    let existing: Skill =
        first_or_404("Skill", skill_id_value, skills::table.find(skill_id_value))?;

    require_owned_resume(user_id_value, existing.resume_id)?;

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
