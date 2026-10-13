use diesel::prelude::*;
use domain::models::{NewSkill, NewSkillRequest, Resume, Skill};
use infrastructure::establish_connection;

use crate::{
    error::ApplicationError,
    realtime::{ResumeChangePublisher, ResumeChangedAction, SectionType},
    resume::common::{app_err_from_diesel_err, find_resume, require_owner},
};

pub fn create_skill(
    user_id_value: i32,
    resume_id_value: i32,
    payload: NewSkillRequest,
    publisher: &dyn ResumeChangePublisher,
) -> Result<Skill, ApplicationError> {
    use domain::schema::skills;

    let existing_resume: Resume = find_resume(resume_id_value)?;

    require_owner(&existing_resume, user_id_value)?;

    let new_skill = NewSkill {
        resume_id: resume_id_value,
        skill_name: payload.skill_name,
        confidence_percentage: payload.confidence_percentage,
        display_order: payload.display_order,
    };

    match diesel::insert_into(skills::table)
        .values(&new_skill)
        .get_result::<Skill>(&mut establish_connection())
    {
        Ok(skill) => {
            publisher.resume_changed(
                resume_id_value,
                ResumeChangedAction::Updated(SectionType::Skills),
            );
            Ok(skill)
        }
        Err(err) => Err(app_err_from_diesel_err(err)),
    }
}
