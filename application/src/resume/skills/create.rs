use diesel::prelude::*;
use domain::models::{NewSkill, NewSkillRequest, Resume, Skill};
use infrastructure::establish_connection;

use crate::error::ApplicationError;

pub fn create_skill(
    user_id_value: i32,
    resume_id_value: i32,
    payload: NewSkillRequest,
) -> Result<Skill, ApplicationError> {
    use domain::schema::resumes;
    use domain::schema::skills;

    let existing_resume: Resume = match resumes::table
        .find(resume_id_value)
        .first(&mut establish_connection())
    {
        Ok(r) => r,
        Err(diesel::result::Error::NotFound) => {
            return Err(ApplicationError::NotFound(format!(
                "Resume with id {} not found",
                resume_id_value
            )));
        }
        Err(err) => return Err(ApplicationError::Internal(format!("Database error - {}", err))),
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
        Err(err) => Err(ApplicationError::Internal(format!("Database error - {}", err))),
    }
}
