use diesel::prelude::*;
use domain::models::{Skill, UpdateSkill};

use crate::{
    error::ApplicationError,
    realtime::{ResumeChangePublisher, ResumeChangedAction, SectionType},
    resume::common::{first_or_404, require_owned_resume, run_published},
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

    run_published(
        "Skill",
        skill_id_value,
        existing.resume_id,
        ResumeChangedAction::Updated(SectionType::Skills),
        publisher,
        |conn| {
            diesel::update(skills::table.find(skill_id_value))
                .set(&payload)
                .get_result::<Skill>(conn)
        },
    )
}
