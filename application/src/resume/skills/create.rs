use diesel::prelude::*;
use domain::models::{NewSkill, NewSkillRequest, Skill};

use crate::{
    error::ApplicationError,
    realtime::{ResumeChangePublisher, ResumeChangedAction, SectionType},
    resume::common::{require_owned_resume, run_published},
};

pub fn create_skill(
    user_id_value: i32,
    resume_id_value: i32,
    payload: NewSkillRequest,
    publisher: &dyn ResumeChangePublisher,
) -> Result<Skill, ApplicationError> {
    use domain::schema::skills;

    require_owned_resume(user_id_value, resume_id_value)?;

    let new_skill = NewSkill {
        resume_id: resume_id_value,
        skill_name: payload.skill_name,
        confidence_percentage: payload.confidence_percentage,
        display_order: payload.display_order,
    };

    run_published(
        "Skill",
        0,
        resume_id_value,
        ResumeChangedAction::Updated(SectionType::Skills),
        publisher,
        |conn| {
            diesel::insert_into(skills::table)
                .values(&new_skill)
                .get_result::<Skill>(conn)
        },
    )
}
