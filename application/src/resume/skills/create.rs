use diesel::prelude::*;
use domain::models::{NewSkill, NewSkillRequest, Resume, Skill};
use infrastructure::establish_connection;

use crate::{
    error::ApplicationError,
    resume::common::{app_err_from_diesel_err, find_resume},
};

pub fn create_skill(
    user_id_value: i32,
    resume_id_value: i32,
    payload: NewSkillRequest,
) -> Result<Skill, ApplicationError> {
    use domain::schema::skills;

    let existing_resume: Resume = match find_resume(resume_id_value) {
        Ok(r) => r,
        Err(err) => return Err(err),
    };

    match existing_resume.created_by {
        Some(owner) if owner == user_id_value => {}
        Some(_) | None => {
            return Err(ApplicationError::Forbidden);
        }
    }

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
        Ok(skill) => Ok(skill),
        Err(err) => Err(app_err_from_diesel_err(err)),
    }
}
